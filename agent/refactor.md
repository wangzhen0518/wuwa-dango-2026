现计划对这个项目进行重构，核心重构点为：
1. 将 Rc<RefCell<Dango>> 转换为 Dango，其中 Dango 由
    ```rust
    pub enum Dango {
        Denia(Denia),
        Sigrika(Sigrika),
        Hiyuki(Hiyuki),
        Cartethyia(Cartethyia),
        Phoebe(Phoebe),
        LuukHerssen(LuukHerssen),
        BuDaWang(BuDaWang),
    }
    ```
    变为
    ```rust
    pub enum Dango {
        Denia(Rc<RefCell<Denia>>),
        Sigrika(Rc<RefCell<Sigrika>>),
        ...
    }
    ```
2. 以 Denia 为例，为 Rc<RefCell<Denia>> 实现 Run，而不是为 Denia 实现 Run

我希望实现一个宏 impl_run_for_dango_helper，使用形式为：
impl_run_for_dango_helper!(
    reset(&self),
    roll<R>(&self, rng: &mut R),
    ...
)

在 impl Run for Dango 中展开后变为当前的实现的形式:
    fn reset(&self) {
        match self {
            Dango::Denia(ref_cell) => ref_cell.reset(),
            Dango::Sigrika(ref_cell) => ref_cell.reset(),
            Dango::Hiyuki(ref_cell) => ref_cell.reset(),
            Dango::Cartethyia(ref_cell) => ref_cell.reset(),
            Dango::Phoebe(ref_cell) => ref_cell.reset(),
            Dango::LuukHerssen(ref_cell) => ref_cell.reset(),
            Dango::BuDaWang(ref_cell) => ref_cell.reset(),
        }
    }

    fn roll<R>(&self, rng: &mut R)
    where
        R: Rng + ?Sized,
    {
        match self {
            Dango::Denia(ref_cell) => ref_cell.roll(rng),
            Dango::Sigrika(ref_cell) => ref_cell.roll(rng),
            Dango::Hiyuki(ref_cell) => ref_cell.roll(rng),
            Dango::Cartethyia(ref_cell) => ref_cell.roll(rng),
            Dango::Phoebe(ref_cell) => ref_cell.roll(rng),
            Dango::LuukHerssen(ref_cell) => ref_cell.roll(rng),
            Dango::BuDaWang(ref_cell) => ref_cell.roll(rng),
        }
    }
 
即

```rust
impl Run for Dango {
    impl_run_for_dango_helper!(
        reset(&self),
        roll<R>(&self, rng: &mut R),
        ...
    )
}
```
展开为
```rust
impl Run for Dango {
    fn reset(&self) {
        match self {
            Dango::Denia(ref_cell) => ref_cell.reset(),
            Dango::Sigrika(ref_cell) => ref_cell.reset(),
            Dango::Hiyuki(ref_cell) => ref_cell.reset(),
            Dango::Cartethyia(ref_cell) => ref_cell.reset(),
            Dango::Phoebe(ref_cell) => ref_cell.reset(),
            Dango::LuukHerssen(ref_cell) => ref_cell.reset(),
            Dango::BuDaWang(ref_cell) => ref_cell.reset(),
        }
    }

    fn roll<R>(&self, rng: &mut R)
    where
        R: Rng + ?Sized,
    {
        match self {
            Dango::Denia(ref_cell) => ref_cell.roll(rng),
            Dango::Sigrika(ref_cell) => ref_cell.roll(rng),
            Dango::Hiyuki(ref_cell) => ref_cell.roll(rng),
            Dango::Cartethyia(ref_cell) => ref_cell.roll(rng),
            Dango::Phoebe(ref_cell) => ref_cell.roll(rng),
            Dango::LuukHerssen(ref_cell) => ref_cell.roll(rng),
            Dango::BuDaWang(ref_cell) => ref_cell.roll(rng),
        }
    }
}
```

请完成该宏的实现