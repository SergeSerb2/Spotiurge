import json
import threading
import unittest
import urllib.error
import urllib.request
from pathlib import Path
from tempfile import TemporaryDirectory
from http.server import ThreadingHTTPServer

from unittest.mock import patch
from server import Store, make_handler, recommend, same_track, RateLimited, BoundedServer, MAX_BYTES, main


class PrivateCloudTests(unittest.TestCase):
    def setUp(self):
        self.directory = TemporaryDirectory(dir=Path(__file__).parent)
        self.path = Path(self.directory.name) / "state.sqlite3"
        self.store = Store(self.path)
        self.server = ThreadingHTTPServer(("127.0.0.1", 0), make_handler(self.store, "dummy-private-test-token"))
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        self.url = f"http://127.0.0.1:{self.server.server_port}"

    def tearDown(self):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join()
        self.directory.cleanup()

    def request(self, method="GET", document=None, revision=None, token="dummy-private-test-token"):
        headers = {"Authorization": "Bearer " + token}
        if revision is not None:
            headers["If-Match"] = str(revision)
        data = None if document is None else json.dumps(document, ensure_ascii=False, separators=(",", ":")).encode()
        request = urllib.request.Request(self.url + "/v1/state", data=data, headers=headers, method=method)
        try:
            with urllib.request.urlopen(request) as response:
                return response.status, json.load(response)
        except urllib.error.HTTPError as error:
            with error:
                return error.code, json.load(error)

    def test_feedback_cutoff_tombstone_round_trips_in_the_version_one_schema(self):
        document = {"version": 1, "records": {"feedback:retention": {
            "stamp": {"counter": 1600, "device": "a" * 32}, "value": None}}}
        status, stored = self.request("PUT", document, 0)
        self.assertEqual(status, 200)
        self.assertEqual(stored["revision"], 1)
        self.assertEqual(self.request()[1]["document"], document)

    def test_unauthorized_requests_cannot_read_or_replace_state(self):
        self.assertEqual(self.request(token="wrong")[0], 401)
        self.assertEqual(self.request("PUT", {}, 0, "wrong")[0], 401)
        self.assertEqual(self.store.read()["revision"], 0)

    def test_startup_requires_the_desktop_ascii_token_bounds(self):
        for token in ["x" * 31, "x" * 257, "é" * 32, "界" * 32]:
            with self.subTest(length=len(token), ascii=token.isascii()):
                with patch.dict("server.os.environ", {"SPOTIURGE_CLOUD_TOKEN": token}, clear=True), \
                        patch("server.os.umask"), patch("server.Store") as store, \
                        patch("server.BoundedServer") as server:
                    with self.assertRaisesRegex(SystemExit, "32–256 ASCII"):
                        main()
                    store.assert_not_called()
                    server.assert_not_called()
        for length in [32, 256]:
            with self.subTest(length=length):
                with patch.dict("server.os.environ", {
                        "SPOTIURGE_CLOUD_TOKEN": "x" * length,
                        "CLI_PROXY_BASE_URL": "https://proxy.invalid/v1",
                        "CLI_PROXY_API_KEY": "dummy-test-key",
                    }, clear=True), patch("server.os.umask"), \
                        patch("server.Store"), patch("server.BoundedServer") as server:
                    main()
                    server.return_value.serve_forever.assert_called_once()

    def test_compare_and_swap_and_restart_preserve_acknowledged_state(self):
        document = {"version": 1, "records": {"taste": {"stamp": {"counter": 1, "device": "a" * 32}, "value": {"kind": "taste", "text": "warm strings"}}}}
        self.assertEqual(self.request("PUT", document, 0)[0], 200)
        self.assertEqual(self.request("PUT", {"version": 1, "records": {}}, 0)[0], 409)
        self.assertEqual(self.request()[1]["document"], document)
        self.assertEqual(Store(self.path).read()["document"], document)

    def test_unknown_schema_and_poisoned_clock_are_rejected(self):
        for version in [2, True, 1.0, "1"]:
            self.assertEqual(self.request("PUT", {"version": version, "records": {}}, 0)[0], 400)
        document = {"version": 1, "records": {"taste": {"stamp": {"counter": 2**64 - 1, "device": "a" * 32}, "value": None}}}
        self.assertEqual(self.request("PUT", document, 0)[0], 400)
        self.assertEqual(self.store.read()["revision"], 0)

    def test_byte_limits_and_revision_overflow_cannot_poison_state(self):
        document = {"version": 1, "records": {"mix:" + "é" * 100: {"stamp": {"counter": 1, "device": "a" * 32}, "value": None}}}
        self.assertEqual(self.request("PUT", document, 0)[0], 400)
        self.assertEqual(self.request("PUT", {"version": 1, "records": {}}, 2**100)[0], 400)
        self.assertEqual(self.store.read()["revision"], 0)

    def test_near_limit_compact_utf8_document_round_trips_without_expansion(self):
        record = {"stamp": {"counter": 1, "device": "a" * 32}, "value": {
            "kind": "history", "prompt": "é" * 2000,
            "suggestions": [{"title": "é" * 150, "artist": "界" * 100,
                "reason": "é" * 300} for _ in range(12)]}}
        document = {"version": 1, "records": {f"history:{i}": record for i in range(55)}}
        compact = json.dumps(document, ensure_ascii=False, separators=(",", ":")).encode()
        self.assertLessEqual(len(compact), MAX_BYTES)
        self.assertGreater(len(json.dumps(document).encode()), MAX_BYTES)
        self.assertEqual(self.request("PUT", document, 0)[0], 200)
        self.assertEqual(self.request()[1]["document"], document)
        self.assertEqual(Store(self.path).read()["document"], document)
        with urllib.request.urlopen(urllib.request.Request(self.url + "/v1/state",
                headers={"Authorization": "Bearer dummy-private-test-token"})) as response:
            self.assertLessEqual(len(response.read()), MAX_BYTES + 200)
        oversized = {"version": 1, "records": {f"history:{i}": record for i in range(56)}}
        self.assertGreater(len(json.dumps(oversized, ensure_ascii=False, separators=(",", ":")).encode()), MAX_BYTES)
        with self.assertRaises(ValueError):
            self.store.write(1, oversized)
        request = urllib.request.Request(self.url + "/v1/state", data=b"{}", method="PUT",
            headers={"Authorization": "Bearer dummy-private-test-token", "If-Match": "1",
                "Content-Length": str(MAX_BYTES + 1)})
        with self.assertRaises(urllib.error.HTTPError) as error:
            urllib.request.urlopen(request)
        self.assertEqual(error.exception.code, 400)
        error.exception.close()
        self.assertEqual(self.store.read()["revision"], 1)

    def test_proxy_rate_limit_does_not_attempt_another_subscription_model(self):
        error = urllib.error.HTTPError("https://proxy.invalid/v1/chat/completions", 429, "rate limited", {}, None)
        with patch("server.urllib.request.OpenerDirector.open", side_effect=error) as call:
            with self.assertRaises(RateLimited):
                recommend({"taste": "warm jazz"}, {"models": ["primary", "fallback"], "proxy_url": "https://proxy.invalid/v1", "proxy_key": "dummy"})
            self.assertEqual(call.call_count, 1)

    def test_proxy_failure_never_falls_back_to_a_heavier_model(self):
        with patch("server.urllib.request.OpenerDirector.open", side_effect=TimeoutError) as call:
            self.assertIsNone(recommend({"taste": "warm jazz"}, {
                "models": ["gpt-6.1-sol", "gpt-6-luna"],
                "proxy_url": "https://proxy.invalid/v1", "proxy_key": "dummy"}))
            self.assertEqual(call.call_count, 1)
            self.assertEqual(json.loads(call.call_args.args[0].data)["model"], "gpt-6-luna")

    def test_unexpected_secret_fields_are_rejected_before_committing(self):
        document = {"version": 1, "records": {"taste": {"stamp": {"counter": 1, "device": "a" * 32}, "value": {"kind": "taste", "text": "jazz", "spotify_token": "dummy"}}}}
        self.assertEqual(self.request("PUT", document, 0)[0], 400)
        self.assertEqual(self.store.read()["revision"], 0)


    def test_feedback_alone_recommends_without_a_taste_prompt_or_uris(self):
        captured = []
        feedback = [{"kind": "feedback", "uri": "spotify:track:" + "a" * 22, "title": "Prayer", "artist": "Prospa", "rating": "love"},
                    {"kind": "feedback", "uri": "spotify:track:" + "b" * 22, "title": "Saving Up", "artist": "Dom Dolla", "rating": "less"}]
        answer = {"suggestions": [{"title": "Prayer", "artist": "prospa", "reason": "repeat"},
                                  {"title": "Imagination", "artist": "Gorgon City", "reason": "warm vocal house"},
                                  {"title": "imagination", "artist": "GORGON CITY", "reason": "duplicate"}]}
        result = self.recommend_with({"taste": "  ", "feedback": feedback, "exploration": "adventurous"}, answer, captured)
        self.assertEqual(captured[0]["model"], "gpt-6-luna")
        self.assertEqual(result["model"], "gpt-6-luna")
        self.assertEqual([s["title"] for s in result["suggestions"]], ["Imagination"])
        prompt = json.dumps(captured[0]["messages"])
        self.assertNotIn("spotify:track", prompt)
        self.assertIn("does not know", prompt)
        self.assertIn("primary credited artist", prompt)
        self.assertEqual(json.loads(captured[0]["messages"][1]["content"])["feedback"][0], {"title": "Prayer", "artist": "Prospa", "rating": "love"})

    def test_empty_input_and_unknown_exploration_are_rejected_before_ai(self):
        config = {"models": ["primary"], "proxy_url": "https://proxy.invalid/v1", "proxy_key": "dummy"}
        with patch("server.urllib.request.OpenerDirector.open") as call:
            for body in [{"taste": ""}, {"taste": " ", "feedback": []}, {}, {"taste": "jazz", "exploration": "wild"},
                         {"taste": "jazz", "exploration": ["balanced"]}, {"taste": "jazz", "exploration": "Balanced"}]:
                with self.assertRaises(ValueError):
                    recommend(body, config)
            self.assertEqual(call.call_count, 0)
        captured = []
        self.recommend_with({"taste": "jazz"}, {"suggestions": [{"title": "So What", "artist": "Miles Davis", "reason": "modal"}]}, captured)
        self.assertIn("about half close matches", captured[0]["messages"][0]["content"])

    def test_primary_or_guest_credits_cannot_repeat_rated_collaborations(self):
        feedback = [
            {"title": "Butterflies", "artist": "Skrillex, Starrah, Four Tet", "rating": "less"},
            {"title": "See You Again", "artist": "Tyler, The Creator, Kali Uchis", "rating": "love"},
        ]
        suggestions = [
            {"title": "  BUTTERFLIES ", "artist": "Skrillex", "reason": "primary repeat"},
            {"title": "Butterflies", "artist": "Four Tet", "reason": "guest repeat"},
            {"title": "See You Again", "artist": "Tyler, The Creator", "reason": "comma name repeat"},
            {"title": "See You Again", "artist": "Kali Uchis", "reason": "guest repeat"},
            {"title": "Butterflies", "artist": "Skrillexx", "reason": "distinct credit"},
            {"title": "Butterflies (Remix)", "artist": "Skrillex", "reason": "distinct version"},
            {"title": "New Song", "artist": "Artist A, Artist B", "reason": "new"},
            {"title": "New Song", "artist": "Artist A", "reason": "same collaboration repeated"},
        ]
        captured = []
        result = self.recommend_with({"feedback": feedback}, {"suggestions": suggestions}, captured)
        self.assertEqual([(s["title"], s["artist"]) for s in result["suggestions"]], [
            ("Butterflies", "Skrillexx"), ("Butterflies (Remix)", "Skrillex"), ("New Song", "Artist A, Artist B")])
        self.assertEqual(captured[0]["model"], "gpt-6-luna")

    def test_featured_titles_cannot_repeat_rated_tracks_but_versions_remain_distinct(self):
        feedback = [
            {"title": "Stay (feat. Alessia Cara)", "artist": "Zedd, Alessia Cara", "rating": "less"},
            {"title": "Butterflies [with Starrah & Four Tet]", "artist": "Skrillex, Starrah, Four Tet", "rating": "love"},
            {"title": "See You Again feat. Kali Uchis", "artist": "Tyler, The Creator, Kali Uchis", "rating": "less"},
        ]
        suggestions = [
            {"title": "Stay", "artist": "Zedd", "reason": "repeat without credit"},
            {"title": "Stay [with Alessia Cara]", "artist": "Zedd", "reason": "alternate credit"},
            {"title": "Butterflies", "artist": "Skrillex", "reason": "two credited guests"},
            {"title": "See You Again", "artist": "Tyler, The Creator", "reason": "comma name"},
            {"title": "Stay (Remix)", "artist": "Zedd", "reason": "different version"},
            {"title": "Stay (Live)", "artist": "Zedd", "reason": "different version"},
            {"title": "Stay (feat. Unknown Artist)", "artist": "Zedd", "reason": "uncredited guest"},
        ]
        captured = []
        result = self.recommend_with({"feedback": feedback}, {"suggestions": suggestions}, captured)
        self.assertEqual([s["title"] for s in result["suggestions"]], [
            "Stay (Remix)", "Stay (Live)", "Stay (feat. Unknown Artist)"])
        self.assertEqual(captured[0]["model"], "gpt-6-luna")

    def test_title_credit_folding_is_symmetric_and_keeps_versions_and_meaningful_titles(self):
        for credited in ["Song (feat. Guest)", "Song [with Guest]", "Song ft. Guest",
                "Song (featuring Guest)", "Song (feat. Guest) (Extended Mix)"]:
            base = "Song (Extended Mix)" if "Extended Mix" in credited else "Song"
            left = {"title": credited, "artist": "Primary, Guest"}
            right = {"title": base, "artist": "Primary"}
            self.assertTrue(same_track(left, right), credited)
            self.assertTrue(same_track(right, left), credited)
        for title in ["Song (Remix)", "Song (Live)", "Song (Edit)", "Song (Extended Mix)",
                "Song with Guest", "Song (feat. Guest Extra)", "Song (feat. Stranger)"]:
            self.assertFalse(same_track({"title": title, "artist": "Primary, Guest"},
                {"title": "Song", "artist": "Primary"}), title)
        self.assertTrue(same_track({"title": "Don’t Go [with Guest]", "artist": "Primary, Guest"},
            {"title": "Don't Go", "artist": "Primary"}))
        self.assertTrue(same_track({"title": "Song (feat. Simon & Garfunkel)", "artist": "Primary, Simon & Garfunkel"},
            {"title": "Song", "artist": "Primary"}))

    def test_server_busy_and_model_rate_limits_report_distinct_codes(self):
        started, release = threading.Event(), threading.Event()

        def slow(body, config):
            started.set()
            release.wait(5)
            raise RateLimited

        server = BoundedServer(("127.0.0.1", 0), make_handler(self.store, "dummy-private-test-token", {"models": []}))
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        url = f"http://127.0.0.1:{server.server_port}/v1/recommendations"

        def post(results):
            request = urllib.request.Request(url, data=b'{"taste": "jazz"}', method="POST",
                headers={"Authorization": "Bearer dummy-private-test-token", "Content-Type": "application/json"})
            try:
                with urllib.request.urlopen(request) as response:
                    results.append((response.status, json.load(response)))
            except urllib.error.HTTPError as error:
                with error:
                    results.append((error.code, json.load(error)))

        try:
            with patch("server.recommend", side_effect=slow):
                first = []
                worker = threading.Thread(target=post, args=(first,))
                worker.start()
                self.assertTrue(started.wait(5))
                second = []
                post(second)
                release.set()
                worker.join(5)
            self.assertEqual(second[0][0], 429)
            self.assertEqual(second[0][1]["code"], "busy")
            self.assertEqual(first[0][0], 429)
            self.assertEqual(first[0][1]["code"], "rate_limited")
        finally:
            release.set()
            server.shutdown()
            server.server_close()
            thread.join()

    def recommend_with(self, body, answer, captured):
        class Reply:
            def __init__(self, request):
                captured.append(json.loads(request.data))

            def __enter__(self):
                return self

            def __exit__(self, *args):
                return False

            def read(self, limit):
                return json.dumps({"choices": [{"message": {"content": json.dumps(answer)}}]}).encode()

        with patch("server.urllib.request.OpenerDirector.open", side_effect=lambda request, timeout: Reply(request)):
            # An old deployment/configuration cannot restore an expensive model.
            return recommend(body, {"models": ["gpt-6.1-sol", "gpt-6-luna"], "proxy_url": "https://proxy.invalid/v1", "proxy_key": "dummy"})


if __name__ == "__main__":
    unittest.main()
