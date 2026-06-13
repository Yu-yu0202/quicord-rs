#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CommandScope {
    Global,
    Guild(&'static [&'static str]),
}
