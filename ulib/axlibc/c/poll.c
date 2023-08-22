#include <poll.h>
#include <stdio.h>

#include <axlibc.h>

int poll(struct pollfd *__fds, nfds_t __nfds, int __timeout)
{
    return ax_poll(__fds, __nfds, __timeout);
}
