# Concurrency

## 1. 线程与同步

我们首先了解 Rust 中的线程，即`std::thread::spawn` ：

```rust
pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
```

它的 signature 中定义了两个泛型变量， `F` 第一个 trait bound 是一次性闭包，第二个 trait bound 是满足 std::marker::Send trait，表示这个闭包可以在线程间传送，第三个 trait bound 是 `'static` 静态生命周期，表示这个闭包是有所有权的，而不能是借用/引用。

> 注意 Rust 中绝大部分数据结构都实现了 Send ，但有些数据结构特别 impl !Send，比如 `std::rc::Rc` 计数指针，而 `std::sync::Arc` 使用的是原子操作，因此它也实现了 Send。

spawn 函数开辟一条线程之后，返回的是 `JoinHandle<T>`，这个句柄可以理解为主线程与这个新开辟的子线程的汇聚点，主线程需要把子线程 join 进来，等待子线程任务执行完成，才会继续执行。我把这个线程句柄理解为一个报警器，如果线程结束，则给到 join 的主线程一个提醒。

```rust
use std::thread;
let handler = thread::spawn(|| { ... });

handler.join().unwrap();
```

我们说有三种线程同步方式，分别是共享内存、CSP（Channel）和Actor，Rust语言中我们经常使用 Channel 的方式来实现线程同步。我们通过一个小例子学习这种同步方式，

我们使用标准库所提供的 `mpsc::channel` ，这个 channel 名副其实，允许多个生产者线程发送消息，单个消费者线程接受并处理消息。所有 channel 都有发送器 `tx` 和接收器 `rx` ，分别供生产者线程和消费者线程使用，并且所有的 `tx` 和 `rx` 都必须被子线程使用，否则主线程会一直等待。

## 2. 矩阵乘法

矩阵乘法是一个很好的例子去理解并发。两个矩阵相乘，前者的每一行分别乘以后者的每一列，得到结果的每一行。一个简单例子是 $3\times 2$ 乘以 $2\times 3$ 矩阵乘法：

$$
\begin{bmatrix}
x_1, x_2\\
x_3, x_4\\
x_5, x_6
\end{bmatrix}\times
\begin{bmatrix}
y_1, y_2, y_3\\
y_4, y_5, y_6
\end{bmatrix}=
\begin{bmatrix}
x_1*y_1+x_2*y_4, x_1*y_2+x_2*y_5, x_1*y_3+x_2*y_6\\
...
\end{bmatrix}
$$

在编程中，我们可以使用二维数组存储一个矩阵，但是我们也可以使用一维数组存储矩阵以获得更高的数据访问效率。那在 $m\times n$ 矩阵的一维数组表示 $arr$ 中，访问第 $(i, j)$ 个元素应该是 $arr[i*n +j]$

`std::ops::Deref` trait 用于强制类型转换，可以使得 `&T -> &U`（假如 `T: Deref<Target = U>`）。实现这个 trait 可以帮助我们减少很多不必要的 `&*` 操作。
