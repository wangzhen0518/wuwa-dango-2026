#![feature(prelude_import)]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
mod roll {
    use ambassador::{Delegate, delegatable_trait};
    trait Run {
        fn roll(&self) {}
    }
    #[doc(inline)]
    ///A macro to be used by [`ambassador::Delegate`] to delegate [`Run`]
    use _ambassador_impl_Run as ambassador_impl_Run;
    #[doc(hidden)]
    #[allow(non_snake_case)]
    mod ambassador_impl_Run {}
    struct Role;
    impl Run for Role {}
    #[delegate(Run)]
    enum Dango {
        Role(Role),
    }
    #[allow(non_snake_case)]
    mod ambassador_module_Run_for_Dango {
        use super::*;
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        pub trait MatchRun<ambassador_X: Run>: Run {}
        #[allow(non_camel_case_types)]
        impl<ambassador_X: Run, ambassador_Y: Run> MatchRun<ambassador_X>
        for ambassador_Y {}
        impl Run for Dango
        where
            Role: Run,
        {
            #[inline]
            #[allow(unused_braces)]
            fn roll(&self) {
                match self {
                    Dango::Role(inner) => return Run::roll(inner),
                }
            }
        }
    }
}
fn main() {}
