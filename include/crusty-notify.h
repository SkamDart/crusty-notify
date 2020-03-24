#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>

struct ResultINotifyEventCTransport;

struct INotifyC {
  Inotify *ptr;
};

struct ResultINotifyCTransport {
  bool is_ok;
  char *err_msg;
  int err_len;
  INotifyC inotify;
};

extern "C" {

/// Release any resources related to our inotify result instance.
void inotify_destroy(INotifyC inotify_c);

void inotify_destroy_error(char *err);

/// C-FFI Functions
/// Initialize the passed inotify instance to watch `path`.
ResultINotifyCTransport inotify_init(char *c_path);

/// Blocking read on the inotify instance.
ResultINotifyEventCTransport inotify_read_blocking(INotifyC inotify_c);

} // extern "C"
