#[derive(Debug, Default)]
pub(crate) struct Comments {
    pub comments: Vec<Comment>,
}

impl Comments {
    pub(crate) fn add_comment(&mut self, comment: &str, author: &str, response_to: usize) {
        let comment = Comment {
            author: author.to_string(),
            text: comment.to_string(),
            response_to: Some(response_to),
        };
        self.comments.push(comment);
    }
}

#[derive(Debug)]
struct Comment {
    pub author: String,
    pub text: String,
    pub response_to: Option<usize>,
}
