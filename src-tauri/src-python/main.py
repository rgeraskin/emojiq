"""Optimized Python utility functions for Tauri application."""

# don't care about exceptions: show must go on :)
# pylint: disable=broad-exception-caught,bare-except

import json
import logging
import threading
import time
from functools import lru_cache
from pathlib import Path
from typing import Any, Dict, List, Optional, Set

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

_tauri_plugin_functions = [
    "get_emojis",
    "increment_usage",
    "get_keywords",
]  # make these functions callable from UI


class EmojiManager:
    """Optimized emoji manager with caching and efficient search."""

    def __init__(
        self,
        emoji_file_path: str = "src-python/emoji.json",
        ranks_file_name: str = ".emojiq_ranks.json",
    ):
        self.emoji_file_path = emoji_file_path
        self.ranks_file_path = Path.home() / ranks_file_name

        # Data storage
        self._emojis: List[Dict] = []
        self._ranks: Dict[str, int] = {}
        self._keywords: Dict[str, List[str]] = {}
        self._index: Dict[str, Set[str]] = {}

        # Threading for file operations
        self._write_lock = threading.Lock()
        self._pending_writes = False
        self._last_write_time = 0
        self._write_delay = 2.0  # Batch writes for 2 seconds

        # Lazy loading flags
        self._emojis_loaded = False
        self._ranks_loaded = False
        self._keywords_built = False
        self._index_built = False

    def _load_emojis(self) -> None:
        """Load emoji data from JSON file."""
        if self._emojis_loaded:
            return

        try:
            self._emojis = _load_json_data(self.emoji_file_path)
            logger.info("Loaded %d emojis", len(self._emojis))
        except:  # noqa: E722
            logger.error("Failed to load emojis")

        self._emojis_loaded = True

    def _load_ranks(self) -> None:
        """Load usage ranks from file."""
        if self._ranks_loaded:
            return

        try:
            self._ranks = _load_json_data(self.ranks_file_path)
            logger.info("Loaded %d ranks", len(self._ranks))
        except:  # noqa: E722
            logger.error("Failed to load ranks")

        self._ranks_loaded = True

    def _build_keywords(self) -> None:
        """Build optimized keyword mappings."""
        if self._keywords_built:
            return

        self._load_emojis()

        try:
            for emoji_data in self._emojis:
                emoji = emoji_data["emoji"]
                description = (
                    emoji_data.get("description", "").lower().replace("_", " ")
                )
                aliases = emoji_data.get("aliases", [])
                tags = emoji_data.get("tags", [])

                # Create keyword list with description first
                keywords = [description]

                # Add aliases and tags, sorted by length for better matching
                all_keywords = [kw for kw in aliases + tags]
                # Lowercase and replace underscores
                all_keywords = [kw.lower().replace("_", " ") for kw in all_keywords]
                # Sort by length
                all_keywords = sorted(all_keywords, key=len)

                # Remove duplicates and description variants
                seen = {description}

                for keyword in all_keywords:
                    if keyword not in seen:
                        keywords.append(keyword)
                        seen.add(keyword)

                self._keywords[emoji] = keywords

            logger.info("Built keywords for %d emojis", len(self._keywords))

        except Exception as e:
            logger.error("Failed to build keywords: %s", e)
        self._keywords_built = True

    def _build_index(self) -> None:
        """Build search index."""
        if self._index_built:
            return

        self._build_keywords()

        try:

            def index_keyword(keywords: list[str], emoji: str) -> None:
                keywords = [keyword for keyword in keywords if len(keyword) >= 2]
                for keyword in keywords:
                    # Index full keyword
                    self._index.setdefault(keyword, set()).add(emoji)

                    # Index prefixes for partial matching (min length 2)
                    for i in range(2, len(keyword) + 1):
                        prefix = keyword[:i]
                        self._index.setdefault(prefix, set()).add(emoji)

            # Build inverted index: keyword -> set of emojis
            for emoji, keywords in self._keywords.items():
                index_keyword(keywords, emoji)
                for keyword in keywords:
                    words = keyword.replace("-", " ").split(" ")
                    if len(words) > 1:
                        index_keyword(words, emoji)

            logger.info("Built index for %d matches", len(self._index))
        except Exception as e:
            logger.error("Failed to build search index: %s", e)
            raise e
        self._index_built = True

    def _get_top_emojis(self, limit: int = 10) -> List[str]:
        """Get most frequently used emojis."""
        if not self._ranks:
            return []

        top_emojis = sorted(self._ranks.keys(), key=lambda x: self._ranks[x])[:limit]

        logger.debug("Top %d emojis: %s", limit, top_emojis)

        return top_emojis

    def _order_emojis_by_usage(self, emojis: List[str]) -> List[str]:
        """Order emojis by usage frequency."""
        if not self._ranks:
            return emojis

        top_emojis = self._get_top_emojis()

        # Create ordering function
        def get_priority(emoji: str) -> int:
            if emoji in top_emojis:
                # Higher priority for top emojis (negative index for reverse sort)
                return -top_emojis.index(emoji)
            # If not in top emojis, give it lowest priority
            return 1

        return sorted(emojis, key=get_priority)

    def _schedule_write(self) -> None:
        """Schedule a batched write operation."""
        with self._write_lock:
            self._pending_writes = True
            self._last_write_time = time.time()

        # Schedule write in separate thread
        threading.Timer(self._write_delay, self._perform_batched_write).start()

    def _perform_batched_write(self) -> None:
        """Perform batched write if still needed."""
        with self._write_lock:
            if not self._pending_writes:
                return

            # Check if enough time has passed since last update
            if time.time() - self._last_write_time < self._write_delay:
                return

            try:
                with open(self.ranks_file_path, "w", encoding="utf-8") as f:
                    json.dump(self._ranks, f, separators=(",", ":"))
                self._pending_writes = False
                logger.info("Wrote usage ranks to file")
            except Exception as e:
                logger.error("Failed to write ranks: %s", e)

    @lru_cache(maxsize=128)
    def get_emojis(self, filter_word: str = "") -> str:
        """Get filtered emojis as space-separated string."""

        filter_word = filter_word.lower().strip()

        self._build_index()
        self._load_ranks()

        # Handle empty or short filter
        if len(filter_word) < 2:
            # Return all emojis ordered by usage
            emoji_list = [emoji_data["emoji"] for emoji_data in self._emojis]
        else:
            # Use direct keyword search (like original) for correct substring matching
            emoji_list = self._index.get(filter_word, [])

        ordered_emojis = self._order_emojis_by_usage(emoji_list)
        return " ".join(ordered_emojis)

    def get_keywords(self, emoji: str) -> str:
        """Get keywords for an emoji as semicolon-separated string."""
        self._build_keywords()

        keywords = self._keywords.get(emoji, [])
        return ";".join(keywords)

    def increment_usage(self, emoji: str) -> None:
        """Increment usage count for an emoji."""
        self._load_ranks()

        self._ranks[emoji] = self._ranks.get(emoji, 0) + 1

        # Clear cache since usage order changed
        self.get_emojis.cache_clear()

        # Schedule batched write
        self._schedule_write()


# Global instance
_emoji_manager: Optional[EmojiManager] = None


def _load_json_data(file_path) -> Any:
    """Load data from JSON files."""
    data = None
    try:
        with open(file_path, "r", encoding="utf-8") as f:
            data = json.load(f)

    except Exception as e:
        logger.error("Failed to load json data: %s", e)
        raise e

    return data


def _get_manager() -> EmojiManager:
    """Get or create the global emoji manager instance."""
    global _emoji_manager  # pylint: disable=global-statement
    if _emoji_manager is None:
        _emoji_manager = EmojiManager()
    return _emoji_manager


# Public API functions (called from Tauri)
def get_emojis(filter_word: str = "") -> str:
    """Get filtered emojis as space-separated string."""
    return _get_manager().get_emojis(filter_word)


def get_keywords(emoji: str) -> str:
    """Get keywords for an emoji as semicolon-separated string."""
    return _get_manager().get_keywords(emoji)


def increment_usage(emoji: str) -> None:
    """Increment usage count for an emoji."""
    _get_manager().increment_usage(emoji)
