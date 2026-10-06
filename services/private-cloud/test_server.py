import json
import threading
import unittest
import urllib.error
import urllib.request
from pathlib import Path
from tempfile import TemporaryDirectory
from http.server import ThreadingHTTPServer

from unittest.mock import patch
from server import Store, make_handler, recommend, RateLimited


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
        data = None if document is None else json.dumps(document).encode()
        request = urllib.request.Request(self.url + "/v1/state", data=data, headers=headers, method=method)
        try:
            with urllib.request.urlopen(request) as response:
                return response.status, json.load(response)
        except urllib.error.HTTPError as error:
            with error:
                return error.code, json.load(error)

    def test_unauthorized_requests_cannot_read_or_replace_state(self):
        self.assertEqual(self.request(token="wrong")[0], 401)
        self.assertEqual(self.request("PUT", {}, 0, "wrong")[0], 401)
        self.assertEqual(self.store.read()["revision"], 0)

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

    def test_proxy_rate_limit_does_not_attempt_another_subscription_model(self):
        error = urllib.error.HTTPError("https://proxy.invalid/v1/chat/completions", 429, "rate limited", {}, None)
        with patch("server.urllib.request.OpenerDirector.open", side_effect=error) as call:
            with self.assertRaises(RateLimited):
                recommend({"taste": "warm jazz"}, {"models": ["primary", "fallback"], "proxy_url": "https://proxy.invalid/v1", "proxy_key": "dummy"})
            self.assertEqual(call.call_count, 1)

    def test_unexpected_secret_fields_are_rejected_before_committing(self):
        document = {"version": 1, "records": {"taste": {"stamp": {"counter": 1, "device": "a" * 32}, "value": {"kind": "taste", "text": "jazz", "spotify_token": "dummy"}}}}
        self.assertEqual(self.request("PUT", document, 0)[0], 400)
        self.assertEqual(self.store.read()["revision"], 0)


if __name__ == "__main__":
    unittest.main()
