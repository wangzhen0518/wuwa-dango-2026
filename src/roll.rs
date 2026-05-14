use ambassador::{Delegate, delegatable_trait};

#[delegatable_trait]
trait Run {
    fn roll(&self) {}
}

struct Role;
impl Run for Role {}

#[derive(Delegate)]
#[delegate(Run)]
enum Dango {
    Role(Role),
}
