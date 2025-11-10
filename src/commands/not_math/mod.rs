pub mod aegyo;
pub mod random;
pub mod registered_trademark;
pub mod reverse;
pub mod scramble;
pub mod sort;
pub mod spacer;
pub mod title;
pub mod trademark;
pub mod trademarkinator;
pub mod uglify;
pub mod unscramble;

use calcbot_attrs::{Command, Info};

/// A collection of strange commands and utilities.
#[derive(Clone, Command, Info)]
#[info(
    category = "Text",
    aliases = ["notmath", "nm"],
    syntax = [""],
    children = [
        aegyo::Aegyo,
        random::Random,
        registered_trademark::RegisteredTrademark,
        reverse::Reverse,
        scramble::Scramble,
        sort::Sort,
        spacer::Spacer,
        title::Title,
        trademark::Trademark,
        trademarkinator::Trademarkinator,
        uglify::Uglify,
        unscramble::Unscramble,
    ],
)]
pub struct NotMath;
