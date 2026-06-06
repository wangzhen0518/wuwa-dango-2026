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