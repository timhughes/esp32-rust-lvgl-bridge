#ifndef HOST_TRANSPORT_H
#define HOST_TRANSPORT_H

#include "../main/include/bridge_protocol.h"

#ifdef __cplusplus
extern "C" {
#endif

// Reset queues (useful between tests)
void host_transport_reset(void);

// Queue inspectors
size_t host_transport_get_cmd_count(void);
size_t host_transport_get_telem_count(void);

#ifdef __cplusplus
}
#endif

#endif // HOST_TRANSPORT_H
