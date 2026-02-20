# Rust 完整特性展示

这是一个包含各种 Markdown 元素的测试文档。

## 代码块测试

下面是一个 Rust 代码示例：

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    let sum: i32 = numbers.iter().sum();
    println!("Sum: {}", sum);
}
```

上面的代码展示了向量的使用。

## 表格测试

### 完整表格

| 特性 | 描述 | 优先级 |
|------|------|--------|
| 内存安全 | 编译期检查 | 高 |
| 零成本抽象 | 无运行时开销 | 高 |
| 并发安全 | 防止数据竞争 | 中 |

### 应用场景对比

| 领域 | Rust | C++ | Go |
|------|------|-----|-----|
| 系统编程 | ✓ | ✓ | × |
| Web后端 | ✓ | × | ✓ |
| 嵌入式 | ✓ | ✓ | × |

## 列表测试

### 无序列表

核心特性：
- **所有权系统**：每个值都有唯一所有者
- **借用检查**：编译时验证引用
- **生命周期**：防止悬垂指针
- **模式匹配**：强大的控制流

### 有序列表

学习路径：
1. 基础语法和所有权
2. 结构体和枚举
3. 错误处理
4. 并发编程
5. 异步编程

### 嵌套列表

项目结构：
- src/
  - main.rs
  - lib.rs
  - modules/
    - config.rs
    - utils.rs
- Cargo.toml
- README.md

## 文本样式测试

这是**粗体文字**，这是*斜体文字*，这是`行内代码`。

还可以组合使用：***粗斜体***、**包含`代码`的粗体**。

## 引用测试

> Rust 连续多年在 Stack Overflow 开发者调查中被评为"最受喜爱的编程语言"。
> 
> 它的设计哲学是：安全、速度、并发。

## 多个代码块

Python 示例：

```python
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)
```

JavaScript 示例：

```javascript
const greet = (name) => {
    console.log(`Hello, ${name}!`);
};
```

## 混合内容

Rust 的 `Result<T, E>` 类型用于错误处理：

```rust
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("除数不能为零".to_string())
    } else {
        Ok(a / b)
    }
}
```

使用时可以这样：
- 使用 `match` 进行模式匹配
- 使用 `?` 操作符传播错误
- 使用 `.unwrap()` 或 `.expect()` 处理

## 总结

Rust 是一门现代化的系统编程语言，适合：

1. 追求**极致性能**的开发者
2. 需要**内存安全**保证的项目
3. **并发编程**密集型应用

*开始你的 Rust 之旅吧！开始你的 Rust 之旅吧！开始你的 Rust 之旅吧！开始你的 Rust 之旅吧！开始你的 Rust 之旅吧！开始你的 Rust 之旅吧！开始你的 Rust 之旅吧！开始你的 Rust 之旅吧！开始你的 Rust 之旅吧！开始你的 Rust 之旅吧！开始你的 Rust 之旅吧！开始你的 Rust 之旅吧！开始你的 Rust 之旅吧！开始你的 Rust 之旅吧！开始你的 Rust 之旅吧！开始你的 Rust 之旅吧！*
