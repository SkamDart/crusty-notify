#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>

struct INotifyC {
  Inotify *ptr;
};

struct ResultCInotifyTransport {
  bool is_ok;
  char *err_msg;
  int err_len;
  INotifyC inotify;
};

extern "C" {

/// Release any resources related to our inotify result instance.
void inotify_destroy(ResultCInotifyTransport _inotify);

/// C-FFI Functions
/// Initialize the passed inotify instance to watch `path`.
ResultCInotifyTransport inotify_init(char *c_path);

/// Blocking read on the inotify instance.
void inotify_read_blocking(INotifyC inotify);

} // extern "C"
