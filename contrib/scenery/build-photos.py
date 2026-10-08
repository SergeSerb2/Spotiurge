#!/usr/bin/env python3
"""Builds assets/scenery/photos.tsv and catalog.tsv from T3 Pretty's scenery.

Usage: build-photos.py <t3-pretty>/apps/web/src/scenery [cache-dir]

The seed pools become photos.tsv, the photos Spotiurge shows before (or
without) its own Unsplash searches. Each photo is reduced to what Spotiurge
needs: its set, Unsplash id, raw image path and query (with Unsplash's ixid,
which also addresses the download ping), place, photographer, and two colours measured from a small
thumbnail: the plain mean and the same weighted "accent" that
src/images.rs computes for album covers. Comparing like with like is what
lets a page pick the photo that best matches its cover.

catalog.ts becomes catalog.tsv: each set's places and the search queries
the app's own refresh sends to Unsplash.

Thumbnails are cached (default: target/scenery-thumbs) so reruns are cheap.
Needs Pillow.
"""

import concurrent.futures
import json
import os
import re
import sys
import urllib.parse
import urllib.request
from io import BytesIO

from PIL import Image

SETS = [
    ("world-scenery", "seedPool.json"),
    ("night-cities", "seeds/night-cities.json"),
    ("deep-forest", "seeds/deep-forest.json"),
    ("night-sky", "seeds/night-sky.json"),
    ("grand-buildings", "seeds/grand-buildings.json"),
]


def accent(pixels):
    """A port of images::art_colors' accent: the heaviest 16-level bucket,
    weighted toward vivid mid-tones."""
    buckets = {}
    for r, g, b in pixels:
        hi, lo = max(r, g, b), min(r, g, b)
        saturation = 0.0 if hi == 0 else (hi - lo) / hi
        lightness = (hi + lo) / 510.0
        weight = (1.0 + saturation * 6.0) * max(1.0 - abs(lightness - 0.5) * 1.4, 0.05)
        weight = int(weight * 100.0)
        bucket = buckets.setdefault((r >> 4, g >> 4, b >> 4), [0, 0, 0, 0])
        bucket[0] += weight
        bucket[1] += r * weight
        bucket[2] += g * weight
        bucket[3] += b * weight
    weight, r, g, b = max(buckets.values(), key=lambda bucket: bucket[0])
    return (r // weight, g // weight, b // weight)


def colours(data):
    image = Image.open(BytesIO(data)).convert("RGB")
    image.thumbnail((48, 48))
    pixels = list(image.getdata())
    mean = tuple(sum(channel) // len(pixels) for channel in zip(*pixels))
    return mean, accent(pixels)


def thumb(path, cache):
    file = os.path.join(cache, path.split("?")[0].strip("/").replace("/", "_") + ".jpg")
    if not os.path.exists(file):
        url = f"https://images.unsplash.com{path.split('?')[0]}?w=96&fm=jpg&q=70&fit=max"
        with urllib.request.urlopen(url, timeout=30) as response:
            data = response.read()
        with open(file + ".part", "wb") as out:
            out.write(data)
        os.replace(file + ".part", file)
    with open(file, "rb") as source:
        return source.read()


def clean(text):
    return " ".join(str(text or "").split())


CATALOGS = {
    "WORLD_SCENERY_CATALOG": "world-scenery",
    "NIGHT_CITIES_CATALOG": "night-cities",
    "DEEP_FOREST_CATALOG": "deep-forest",
    "NIGHT_SKY_CATALOG": "night-sky",
    "GRAND_BUILDINGS_CATALOG": "grand-buildings",
}


def catalog(source):
    with open(os.path.join(source, "catalog.ts")) as handle:
        text = handle.read()
    string = r'"((?:[^"\\]|\\.)*)"'
    entry = re.compile(r"name:\s*" + string + r",\s*query:\s*" + string)
    rows = []
    parts = re.split(r"export const (\w+)", text)
    for name, body in zip(parts[1::2], parts[2::2]):
        if name in CATALOGS:
            for place, query in entry.findall(body):
                rows.append((CATALOGS[name], json.loads(f'"{place}"'), json.loads(f'"{query}"')))
    return rows


def main():
    source = sys.argv[1]
    cache = sys.argv[2] if len(sys.argv) > 2 else "target/scenery-thumbs"
    os.makedirs(cache, exist_ok=True)
    rows = []
    for set_id, file in SETS:
        with open(os.path.join(source, file)) as handle:
            for photo in json.load(handle)["photos"]:
                url = urllib.parse.urlparse(photo["rawURL"] or photo["heroURL"])
                ixid = urllib.parse.parse_qs(url.query).get("ixid", [""])[0]
                path = url.path + (f"?ixid={ixid}" if ixid else "")
                profile = photo.get("photographerProfileURL") or ""
                handle_name = urllib.parse.urlparse(profile).path.strip("/").lstrip("@")
                rows.append(
                    [set_id, photo["id"], path, clean(photo["name"]),
                     clean(photo["photographerName"]), handle_name]
                )
    with concurrent.futures.ThreadPoolExecutor(16) as pool:
        measured = list(pool.map(lambda row: colours(thumb(row[2], cache)), rows))
    hexed = lambda rgb: "#%02x%02x%02x" % rgb
    out = os.path.join(os.path.dirname(__file__), "../../assets/scenery/photos.tsv")
    with open(out, "w") as tsv:
        tsv.write("# set\tid\tpath\tplace\tphotographer\thandle\tmean\taccent\n")
        for row, (mean, vivid) in zip(rows, measured):
            tsv.write("\t".join(row + [hexed(mean), hexed(vivid)]) + "\n")
    print(f"{len(rows)} photos -> {os.path.normpath(out)}")
    places = catalog(source)
    out = os.path.join(os.path.dirname(__file__), "../../assets/scenery/catalog.tsv")
    with open(out, "w") as tsv:
        tsv.write("# set\tplace\tquery\n")
        for row in places:
            tsv.write("\t".join(clean(part) for part in row) + "\n")
    print(f"{len(places)} places -> {os.path.normpath(out)}")


if __name__ == "__main__":
    main()
