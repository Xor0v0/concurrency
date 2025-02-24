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

spawn 函数开辟一条线程之后，返回的是 `JoinHandle<T>`，这个句柄可以理解为主线程与这个新开辟的子线程的汇聚点，主线程需要把子线程 join 进来，等待子线程任务执行完成，才会继续执行。我把这个线程句柄理解为一个报警器，如果线程结束，则给到 join 的主线程一个提醒。没有 join 的话，主线程则不会等待子线程执行结束。

```rust
use std::thread;
let handler = thread::spawn(|| { ... });

handler.join().unwrap();
```

三种线程同步方式，分别是共享内存、CSP（Channel）和Actor。

若并发编程处理不当，则会产生以下问题：
- 条件竞争：多线程的输出结果依赖于不受控制的事件的时间顺序。由于并发时执行流的不确定性很大，条件竞争问题很容易发生。
- 死锁：两个线程相互等待对方完成。
- 特定情形下出现的 bug。

## 2. 矩阵乘法（Channel）

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

我们使用标准库所提供的 `mpsc::channel` ，顾名思义 channel 就像一个管道，允许多个生产者线程发送消息，单个消费者线程接受并处理消息。所有 channel 都有发送器 `tx` 和接收器 `rx` ，分别供生产者线程和消费者线程使用，并且所有的 `tx` 和 `rx` 都必须被子线程使用，否则主线程会一直等待。

## 3. 并发 HashMap 收集实时信息（共享内存）

这个应用场景中需要注意 Metrics 在多线程中的安全访问。Rust 标准库提供 `Arc<T>` 使得可以多线程安全访问不可变数据，而 `Mutex<T>` 能够对外保持不变性，同时提供「内部可变性」。Mutex 这个数据结构之所以能够做到内部可变，是依赖了 Rust 提供的 UnsafeCell：

```Rust
pub struct Mutex<T> {
    // 实际数据
    data: UnsafeCell<T>,
    // 其他字段: 线程同步
}
```

UnsafeCell 允许在 `&self` 上修改 `T` 的内容，这是 Rust 实现内部可变性的核心。在多线程访问同一资源时，如果使用 `Mutex<T>` 对其进行 lock，则会返回 `MutexGuard<T>`。这个数据结构会持有对资源 `T` 的锁，不允许其他线程访问该资源，当这个数据结构离开作用域被释放后，锁也会被自动释放，其他线程才可以访问该资源。
