// C ABI of the Rust engine in ../src/lib.rs. Keep the two in step.
#pragma once
#include <stddef.h>
#include <stdint.h>

typedef void (*probe_event_cb)(const char *json, void *ctx);
typedef void (*probe_credentials_cb)(const uint8_t *data, size_t len, void *ctx);

typedef struct {
    uint64_t decoded_frames;
    uint64_t rendered_frames;
    uint64_t silent_frames;
    float last_rms;
} ProbeStats;

int32_t probe_start(const char *cache_dir, const char *device_name,
                    probe_event_cb on_event, probe_credentials_cb on_credentials,
                    void *ctx);
// kind 0: access token for the streaming scope; kind 1: stored credential JSON.
int32_t probe_connect(uint32_t kind, const uint8_t *data, size_t len);
int32_t probe_load(const char *context_uri);
// 0 play, 1 pause, 2 next, 3 previous, 4 transfer playback here.
int32_t probe_command(uint32_t command);
size_t probe_render(float *left, float *right, size_t frames);
ProbeStats probe_stats(void);
void probe_flush(void);
// Ends the current Connect session without restoring playback.
void probe_disconnect(void);
