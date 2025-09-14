"""Python utility functions for Tauri application."""

import json
from pathlib import Path

_tauri_plugin_functions = [
    "get_emojis",
    "increment_usage",
    "get_keywords",
]  # make these functions callable from UI

EMOJI_FILE_PATH = "src-python/emoji.json"
RANK_DATA_FILE_NAME = ".emojiq_ranks.json"


def init():
    """Initialize global variables."""

    home_dir = Path.home()
    _ranks_file_path = home_dir / RANK_DATA_FILE_NAME

    if _ranks_file_path.exists():
        with open(_ranks_file_path, "r", encoding="utf-8") as f:
            _ranks = json.load(f)
    else:
        _ranks = {}

    with open(EMOJI_FILE_PATH, "r", encoding="utf-8") as f:
        _emoji_data = json.load(f)

    _emoji_keywords = {}
    for emoji in _emoji_data:
        description = emoji["description"]
        keys = sorted(emoji["aliases"] + emoji["tags"], key=len)

        if description in keys:
            keys.remove(description)

        # sometime keywords contain description, but with underscore
        # remove it
        description_underscore = description.replace(" ", "_")
        if description_underscore in keys:
            keys.remove(description_underscore)

        # remove duplicates
        keys = list(set(keys))

        _emoji_keywords[emoji["emoji"]] = [description] + keys

    return _emoji_data, _emoji_keywords, _ranks, _ranks_file_path


def get_keywords(emoji):
    """Get keywords for an emoji."""
    keys = emoji_keywords[emoji]
    return ";".join(keys)


def order_emojis(emojis):
    """Order emojis by usage."""
    # find top 10 most used emojis
    top_emojis = sorted(ranks, key=lambda x: ranks[x], reverse=True)[:10]
    top_emojis.reverse()
    # order emojis by usage
    return sorted(
        emojis,
        key=lambda x: top_emojis.index(x) if x in top_emojis else 0,
        reverse=True,
    )


def get_emojis(filter_word=""):
    """Load and return a list of emoji characters from json."""
    emojis = []

    if len(filter_word) < 2:
        filter_word = ""

    if filter_word:
        for emoji in emoji_data:
            keywords = emoji["aliases"] + emoji["tags"] + [emoji["description"]]
            for keyword in keywords:
                if filter_word in keyword:
                    emojis.append(emoji["emoji"])
                    break
    else:
        emojis = [x["emoji"] for x in emoji_data]

    return " ".join(order_emojis(emojis))


def increment_usage(emoji):
    """Increment usage of an emoji."""
    ranks[emoji] = ranks.get(emoji, 0) + 1
    dump_ranks()


def dump_ranks():
    """Dump rank data to a file."""

    with open(ranks_file_path, "w", encoding="utf-8") as ranks_file:
        json.dump(ranks, ranks_file)


emoji_data, emoji_keywords, ranks, ranks_file_path = init()

# a = sys.argv[1] if len(sys.argv) > 1 else ""
# # print(get_emojis(a))
# print(get_keywords(a))
