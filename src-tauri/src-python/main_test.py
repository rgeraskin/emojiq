"""Test suite for main.py emoji functionality."""

import json
import os
import tempfile
import time
import unittest

from main import EmojiManager


class TestEmojiManager(unittest.TestCase):
    """Test cases for EmojiManager class."""

    def setUp(self):
        """Set up test fixtures."""
        # Use real emoji.json file from the project
        self.emoji_file = os.path.join(os.path.dirname(__file__), "emoji.json")

        # Sample ranks data for testing
        self.sample_ranks_data = {"🧡": 5, "🎉": 3, "👀": 6, "🐒": 1}

        # Create temporary ranks file
        self.temp_dir = tempfile.mkdtemp()
        self.ranks_file = os.path.join(self.temp_dir, "ranks.json")

        # Write sample ranks data to file
        with open(self.ranks_file, "w", encoding="utf-8") as f:
            json.dump(self.sample_ranks_data, f)

    def tearDown(self):
        """Clean up test fixtures."""
        # Delay to handle file write delay
        time.sleep(3)
        # Remove temporary ranks file
        if os.path.exists(self.ranks_file):
            os.remove(self.ranks_file)
        os.rmdir(self.temp_dir)

    def test_emoji_manager_initialization(self):
        """Test EmojiManager initialization with custom paths."""
        manager = EmojiManager(
            emoji_file_path=self.emoji_file, ranks_file_name=self.ranks_file
        )

        self.assertEqual(manager.emoji_file_path, self.emoji_file)
        self.assertEqual(str(manager.ranks_file_path), self.ranks_file)
        # pylint: disable=protected-access
        self.assertFalse(manager._emojis_loaded)
        self.assertFalse(manager._ranks_loaded)
        self.assertFalse(manager._keywords_built)
        self.assertFalse(manager._index_built)

    def test_load_emojis(self):
        """Test emoji data loading."""
        manager = EmojiManager(emoji_file_path=self.emoji_file)
        # pylint: disable=protected-access
        manager._load_emojis()

        self.assertTrue(manager._emojis_loaded)
        self.assertEqual(manager._emojis[0]["emoji"], "😀")
        self.assertEqual(manager._emojis[1]["emoji"], "😃")
        self.assertEqual(manager._emojis[2]["emoji"], "😄")
        self.assertEqual(manager._emojis[3]["emoji"], "😁")

    def test_load_ranks(self):
        """Test ranks data loading."""
        manager = EmojiManager(ranks_file_name=self.ranks_file)
        # pylint: disable=protected-access
        manager._load_ranks()

        self.assertTrue(manager._ranks_loaded)
        self.assertEqual(len(manager._ranks), len(self.sample_ranks_data))
        self.assertEqual(manager._ranks["👀"], 6)
        self.assertEqual(manager._ranks["🧡"], 5)
        self.assertEqual(manager._ranks["🎉"], 3)
        self.assertEqual(manager._ranks["🐒"], 1)

    def test_build_index(self):
        """Test search index building."""
        manager = EmojiManager(emoji_file_path=self.emoji_file)
        # pylint: disable=protected-access
        manager._build_index()

        self.assertTrue(manager._index_built)

        self.assertIn("📆", manager._index["ca"])
        self.assertIn("📆", manager._index["cal"])
        self.assertIn("📆", manager._index["cale"])
        self.assertIn("📆", manager._index["calen"])
        self.assertIn("📆", manager._index["calend"])
        self.assertIn("📆", manager._index["calenda"])
        self.assertIn("📆", manager._index["calendar"])
        self.assertIn("📆", manager._index["of"])
        self.assertIn("📆", manager._index["off"])
        self.assertIn("📆", manager._index["sc"])
        self.assertIn("📆", manager._index["sch"])
        self.assertIn("📆", manager._index["sche"])
        self.assertIn("📆", manager._index["sched"])
        self.assertIn("📆", manager._index["schedu"])
        self.assertIn("📆", manager._index["schedul"])
        self.assertIn("📆", manager._index["schedule"])
        self.assertIn("📆", manager._index["te"])
        self.assertIn("📆", manager._index["tea"])
        self.assertIn("📆", manager._index["tear"])
        self.assertIn("📆", manager._index["tear-"])
        self.assertIn("📆", manager._index["tear-o"])
        self.assertIn("📆", manager._index["tear-of"])
        self.assertIn("📆", manager._index["tear-off"])
        self.assertIn("📆", manager._index["tear-off "])
        self.assertIn("📆", manager._index["tear-off c"])
        self.assertIn("📆", manager._index["tear-off ca"])
        self.assertIn("📆", manager._index["tear-off cal"])
        self.assertIn("📆", manager._index["tear-off cale"])
        self.assertIn("📆", manager._index["tear-off calen"])
        self.assertIn("📆", manager._index["tear-off calend"])
        self.assertIn("📆", manager._index["tear-off calenda"])
        self.assertIn("📆", manager._index["tear-off calendar"])

    def test_get_emojis_empty_filter(self):
        """Test get_emojis with empty filter."""
        manager = EmojiManager(
            emoji_file_path=self.emoji_file, ranks_file_name=self.ranks_file
        )

        result = manager.get_emojis("")
        emojis = result.split()

        # all top ranked emojis
        self.assertEqual("👀", emojis[0])
        self.assertEqual("🧡", emojis[1])
        self.assertEqual("🎉", emojis[2])
        self.assertEqual("🐒", emojis[3])
        # others in default order
        self.assertEqual("😀", emojis[4])
        self.assertEqual("😃", emojis[5])
        self.assertEqual("😄", emojis[6])
        self.assertEqual("😁", emojis[7])

    def test_get_emojis_with_filter(self):
        """Test get_emojis with search filter."""
        manager = EmojiManager(
            emoji_file_path=self.emoji_file, ranks_file_name=self.ranks_file
        )

        result = manager.get_emojis("monkey")
        emojis = result.split()

        # first is the top ranked emoji
        self.assertEqual(emojis[0], "🐒")
        # then the other emojis
        self.assertIn("🐵", emojis)
        self.assertIn("🙊", emojis)
        self.assertIn("🙈", emojis)
        self.assertIn("🙉", emojis)

    def test_get_keywords(self):
        """Test get_keywords functionality."""
        manager = EmojiManager(emoji_file_path=self.emoji_file)

        keywords = manager.get_keywords("😀")
        keyword_list = keywords.split(";")

        self.assertIn("grinning face", keyword_list)
        self.assertIn("grinning", keyword_list)
        self.assertIn("smile", keyword_list)
        self.assertIn("happy", keyword_list)

    def test_get_keywords_nonexistent_emoji(self):
        """Test get_keywords with non-existent emoji."""
        manager = EmojiManager(emoji_file_path=self.emoji_file)

        # Use a truly non-existent emoji (made up character)
        keywords = manager.get_keywords("🏴‍☠️🦄")  # Complex non-standard emoji
        self.assertEqual(keywords, "")

    def test_increment_usage(self):
        """Test usage increment functionality."""
        manager = EmojiManager(ranks_file_name=self.ranks_file)

        # Initial count for 😀 is 5
        # pylint: disable=protected-access
        manager._load_ranks()
        initial_count = manager._ranks.get("😀", 0)

        # Increment usage
        manager.increment_usage("😀")

        # Should be incremented by 1
        self.assertEqual(manager._ranks["😀"], initial_count + 1)

    def test_increment_usage_new_emoji(self):
        """Test incrementing usage for new emoji."""
        manager = EmojiManager(ranks_file_name=self.ranks_file)

        # Increment usage for emoji not in ranks
        manager.increment_usage("🚀")

        # Should start at 1
        # pylint: disable=protected-access
        self.assertEqual(manager._ranks["🚀"], 1)


if __name__ == "__main__":
    unittest.main()
