# 重构：将 `Rc<RefCell<Dango>>` 转换为 `Dango`（内部持有 `Rc<RefCell<T>>`）

## 目标

1. `Dango` 枚举变体从持有直接 struct 改为持有 `Rc<RefCell<T>>`：
   - 前: `Denia(Denia)` → 后: `Denia(Rc<RefCell<Denia>>)`
2. 为 `Rc<RefCell<Denia>>` 实现 `Run` trait，而非仅为 `Denia` 实现
3. 消除所有外部 `.borrow()` / `.borrow_mut()` 调用

## 架构变更

**重构前:**
```
RefDango = Rc<RefCell<Dango>>
                      └── Dango enum ──match──▶ Denia (直接字段访问)
```

**重构后:**
```
Dango = enum ──match──▶ Denia(Rc<RefCell<Denia>>)
                                          └── borrow_mut() → &mut Denia
```

`Dango` 本身不再被包裹——它成为句柄类型。克隆 `Dango` 会克隆内部的 `Rc`，保持共享所有权。

## 实施步骤

### Phase 1: 添加 `impl Run for Rc<RefCell<T>>`（dangos.rs）

为每个变体类型添加 `impl Run for Rc<RefCell<X>>`，通过 `borrow()`/`borrow_mut()` 委托到内部 struct 的同名方法。**保持原有的 `impl Run for X` 不变**，避免重新借用问题。

### Phase 2: 修改 `Dango` 枚举变体（dangos.rs）

```rust
pub enum Dango {
    Denia(Rc<RefCell<Denia>>),
    Sigrika(Rc<RefCell<Sigrika>>),
    // ...
}
```

保持 `#[delegate(Run)]`——ambassador 会生成 match arm 调用 `Rc<RefCell<T>>::Run` 的方法。

### Phase 3: 更新辅助函数和类型别名（dangos.rs）

- `pub type RefDango = Dango;`——别名保留但现在等同 `Dango` 本身
- `is_budawang(&Dango)` → 直接模式匹配
- `sort_dangos(&mut [Dango])` → 不再需要 `.borrow()`
- 工厂函数：`Dango::Denia(Rc::new(RefCell::new(Denia::new())))`——不再有外层包裹

### Phase 4: 清理 main.rs

- 移除所有 `.borrow()` / `.borrow_mut()` 调用
- 所有方法直接在 `Dango` 上调用（通过 ambassador 委托）

### Phase 5: 清理 track.rs

- 更新导入和类型使用
- 无概念性变更，仅有类型适配

## 关键设计决策

1. **保留 `Run` 在内部 struct 和 `Rc<RefCell<T>>` 两层的实现**——内部实现保持原样（直接访问自己的字段），包装层借用后委托
2. **使用 `ambassador` `#[delegate(Run)]`**——当内部类型是 `Rc<RefCell<T>>` 且 `Run` 已实现时有效
3. **`RefDango` → 保持别名但类型变为 `Dango`**——清楚表达意图

## 影响范围

| 文件 | 变更内容 |
|------|---------|
| `src/dangos.rs` | 新增 7 个 `impl Run for Rc<RefCell<X>>` 块；修改枚举变体；更新 `RefDango`；更新辅助函数；更新工厂函数 |
| `src/main.rs` | 移除所有 `.borrow()`/`.borrow_mut()` 调用；更新类型 |
| `src/track.rs` | 无概念性变更，仅有类型适配 |

## 验证

- `cargo check` 通过
- `cargo run` 正常运行