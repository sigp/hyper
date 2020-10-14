#ifndef _HYPER_H
#define _HYPER_H

#include <stdint.h>
#include <stddef.h>

#define HYPER_ITER_CONTINUE 0

#define HYPER_IO_PENDING 4294967295

#define HYPER_IO_ERROR 4294967294

typedef enum {
  HYPERE_OK,
  HYPERE_INVALID_ARG,
} hyper_code;

typedef enum {
  HYPER_TASK_EMPTY,
  HYPER_TASK_ERROR,
  HYPER_TASK_CLIENTCONN,
  HYPER_TASK_RESPONSE,
  HYPER_TASK_BUF,
} hyper_task_return_type;

typedef struct hyper_executor hyper_executor;

typedef struct hyper_io hyper_io;

typedef struct hyper_task hyper_task;

typedef struct hyper_body hyper_body;

typedef struct hyper_buf hyper_buf;

typedef struct hyper_clientconn hyper_clientconn;

typedef struct hyper_clientconn_options hyper_clientconn_options;

typedef struct hyper_context hyper_context;

typedef struct hyper_headers hyper_headers;

typedef struct hyper_request hyper_request;

typedef struct hyper_response hyper_response;

typedef struct hyper_waker hyper_waker;

typedef int (*hyper_body_foreach_callback)(void*, const hyper_buf*);

typedef int (*hyper_headers_foreach_callback)(void*, const uint8_t*, size_t, const uint8_t*, size_t);

typedef size_t (*hyper_io_read_callback)(void*, hyper_context*, uint8_t*, size_t);

typedef size_t (*hyper_io_write_callback)(void*, hyper_context*, const uint8_t*, size_t);

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/*
 Returns a static ASCII (null terminated) string of the hyper version.
 */
const char *hyper_version(void);

/*
 Free a `hyper_body *`.
 */
void hyper_body_free(hyper_body *body);

/*
 Return a task that will poll the body for the next buffer of data.

 The task value may have different types depending on the outcome:

 - `HYPER_TASK_BUF`: Success, and more data was received.
 - `HYPER_TASK_ERROR`: An error retrieving the data.
 - `HYPER_TASK_EMPTY`: The body has finished streaming data.

 This does not consume the `hyper_body *`, so it may be used to again.
 However, it MUST NOT be used or freed until the related task completes.
 */
hyper_task *hyper_body_data(hyper_body *body);

/*
 Return a task that will poll the body and execute the callback with each
 body chunk that is received.

 The `hyper_buf` pointer is only a borrowed reference, it cannot live outside
 the execution of the callback. You must make a copy to retain it.

 This will consume the `hyper_body *`, you shouldn't use it anymore or free it.
 */
hyper_task *hyper_body_foreach(hyper_body *body, hyper_body_foreach_callback func, void *userdata);

/*
 Get a pointer to the bytes in this buffer.

 This should be used in conjunction with `hyper_buf_len` to get the length
 of the bytes data.

 This pointer is borrowed data, and not valid once the `hyper_buf` is
 consumed/freed.
 */
const uint8_t *hyper_buf_bytes(const hyper_buf *buf);

/*
 Get the length of the bytes this buffer contains.
 */
size_t hyper_buf_len(const hyper_buf *buf);

/*
 Free this buffer.
 */
void hyper_buf_free(hyper_buf *buf);

/*
 Starts an HTTP client connection handshake using the provided IO transport
 and options.

 Both the `io` and the `options` are consumed in this function call.

 The returned `hyper_task *` must be polled with an executor until the
 handshake completes, at which point the value can be taken.
 */
hyper_task *hyper_clientconn_handshake(hyper_io *io, hyper_clientconn_options *options);

/*
 Send a request on the client connection.

 Returns a task that needs to be polled until it is ready. When ready, the
 task yields a `hyper_response *`.
 */
hyper_task *hyper_clientconn_send(hyper_clientconn *conn, hyper_request *req);

/*
 Free a `hyper_clientconn *`.
 */
void hyper_clientconn_free(hyper_clientconn *conn);

/*
 Creates a new set of HTTP clientconn options to be used in a handshake.
 */
hyper_clientconn_options *hyper_clientconn_options_new(void);

/*
 Set the client background task executor.

 This does not consume the `options` or the `exec`.
 */
void hyper_clientconn_options_exec(hyper_clientconn_options *opts, const hyper_executor *exec);

/*
 Construct a new HTTP request.
 */
hyper_request *hyper_request_new(void);

/*
 Free an HTTP request if not going to send it on a client.
 */
void hyper_request_free(hyper_request *req);

/*
 Set the HTTP Method of the request.
 */
hyper_code hyper_request_set_method(hyper_request *req, const uint8_t *method, size_t method_len);

/*
 Set the URI of the request.
 */
hyper_code hyper_request_set_uri(hyper_request *req, const uint8_t *uri, size_t uri_len);

/*
 Gets a reference to the HTTP headers of this request

 This is not an owned reference, so it should not be accessed after the
 `hyper_request` has been consumed.
 */
hyper_headers *hyper_request_headers(hyper_request *req);

/*
 Free an HTTP response after using it.
 */
void hyper_response_free(hyper_response *resp);

/*
 Get the HTTP-Status code of this response.

 It will always be within the range of 100-599.
 */
uint16_t hyper_response_status(const hyper_response *resp);

/*
 Gets a reference to the HTTP headers of this response.

 This is not an owned reference, so it should not be accessed after the
 `hyper_response` has been freed.
 */
hyper_headers *hyper_response_headers(hyper_response *resp);

/*
 Take ownership of the body of this response.

 It is safe to free the response even after taking ownership of its body.
 */
hyper_body *hyper_response_body(hyper_response *resp);

/*
 Iterates the headers passing each name and value pair to the callback.

 The `userdata` pointer is also passed to the callback.

 The callback should return `HYPER_ITER_CONTINUE` to keep iterating, or
 some other value to stop.
 */
void hyper_headers_foreach(const hyper_headers *headers,
                           hyper_headers_foreach_callback func,
                           void *userdata);

/*
 Sets the header with the provided name to the provided value.

 This overwrites any previous value set for the header.
 */
hyper_code hyper_headers_set(hyper_headers *headers,
                             const uint8_t *name,
                             size_t name_len,
                             const uint8_t *value,
                             size_t value_len);

/*
 Adds the provided value to the list of the provided name.

 If there were already existing values for the name, this will append the
 new value to the internal list.
 */
hyper_code hyper_headers_add(hyper_headers *headers,
                             const uint8_t *name,
                             size_t name_len,
                             const uint8_t *value,
                             size_t value_len);

/*
 Create a new IO type used to represent a transport.

 The read and write functions of this transport should be set with
 `hyper_io_set_read` and `hyper_io_set_write`.
 */
hyper_io *hyper_io_new(void);

/*
 Free an unused `hyper_io *`.

 This is typically only useful if you aren't going to pass ownership
 of the IO handle to hyper, such as with `hyper_clientconn_handshake()`.
 */
void hyper_io_free(hyper_io *io);

/*
 Set the user data pointer for this IO to some value.

 This value is passed as an argument to the read and write callbacks.
 */
void hyper_io_set_userdata(hyper_io *io, void *data);

/*
 Set the read function for this IO transport.

 Data that is read from the transport should be put in the `buf` pointer,
 up to `buf_len` bytes. The number of bytes read should be the return value.

 If there is no data currently available, a waker should be claimed from
 the `ctx` and registered with whatever polling mechanism is used to signal
 when data is available later on. The return value should be
 `HYPER_IO_PENDING`.

 If there is an irrecoverable error reading data, then `HYPER_IO_ERROR`
 should be the return value.
 */
void hyper_io_set_read(hyper_io *io, hyper_io_read_callback func);

/*
 Set the write function for this IO transport.

 Data from the `buf` pointer should be written to the transport, up to
 `buf_len` bytes. The number of bytes written should be the return value.

 If no data can currently be written, the `waker` should be cloned and
 registered with whatever polling mechanism is used to signal when data
 is available later on. The return value should be `HYPER_IO_PENDING`.

 Yeet.

 If there is an irrecoverable error reading data, then `HYPER_IO_ERROR`
 should be the return value.
 */
void hyper_io_set_write(hyper_io *io, hyper_io_write_callback func);

/*
 Creates a new task executor.
 */
const hyper_executor *hyper_executor_new(void);

/*
 Frees an executor and any incomplete tasks still part of it.
 */
void hyper_executor_free(const hyper_executor *exec);

/*
 Push a task onto the executor.

 The executor takes ownership of the task, it should not be accessed
 again unless returned back to the user with `hyper_executor_poll`.
 */
hyper_code hyper_executor_push(const hyper_executor *exec, hyper_task *task);

/*
 Polls the executor, trying to make progress on any tasks that have notified
 that they are ready again.

 If ready, returns a task from the executor that has completed.

 If there are no ready tasks, this returns `NULL`.
 */
hyper_task *hyper_executor_poll(const hyper_executor *exec);

/*
 Free a task.
 */
void hyper_task_free(hyper_task *task);

/*
 Takes the output value of this task.

 This must only be called once polling the task on an executor has finished
 this task.

 Use `hyper_task_type` to determine the type of the `void *` return value.
 */
void *hyper_task_value(hyper_task *task);

/*
 Query the return type of this task.
 */
hyper_task_return_type hyper_task_type(hyper_task *task);

/*
 Set a user data pointer to be associated with this task.

 This value will be passed to task callbacks, and can be checked later
 with `hyper_task_userdata`.
 */
void hyper_task_set_userdata(hyper_task *task, void *userdata);

/*
 Retrieve the userdata that has been set via `hyper_task_set_userdata`.
 */
void *hyper_task_userdata(hyper_task *task);

/*
 Copies a waker out of the task context.
 */
hyper_waker *hyper_context_waker(hyper_context *cx);

/*
 Free a waker that hasn't been woken.
 */
void hyper_waker_free(hyper_waker *waker);

/*
 Free a waker that hasn't been woken.
 */
void hyper_waker_wake(hyper_waker *waker);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* _HYPER_H */
