use crate::parse::Comment;
use bstr::{BString, ByteVec};
use std::borrow::Cow;
use std::fmt::Display;

impl Comment<'_> {
    /// Turn this instance into a fully owned one with `'static` lifetime.
    #[must_use]
    pub fn to_owned(&self) -> Comment<'static> {
        Comment {
            comment_tag: self.comment_tag,
            comment: Cow::Owned(self.comment.as_ref().into()),
        }
    }
}

impl Display for Comment<'_> {
    /// Note that this is a best-effort attempt at printing an comment. If
    /// there are non UTF-8 values in your config, this will _NOT_ render
    /// as read.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.comment_tag.fmt(f)?;
        if let Ok(s) = std::str::from_utf8(&self.comment) {
            s.fmt(f)
        } else {
            write!(f, "{:02x?}", self.comment)
        }
    }
}

impl From<Comment<'_>> for BString {
    fn from(c: Comment<'_>) -> Self {
        c.into()
    }
}

impl From<&Comment<'_>> for BString {
    fn from(c: &Comment<'_>) -> Self {
        let mut values = BString::from(vec![c.comment_tag]);
        values.push_str(c.comment.as_ref());
        values
    }
}
