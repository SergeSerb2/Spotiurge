// Development-only import of the playback probe's engine ABI. Swift calls it
// only when SPOTIURGE_ENGINE is defined (build.sh links the staticlib then).
#include "../../../probes/ios-playback/engine/include/probe_engine.h"
