use blog_domain::model::comments::{
    comment::{Comment, NewComment, UpdateComment},
    i_comment_repository::ICommentRepository,
};

pub struct CommentAppService<T: ICommentRepository> {
    repository: T,
}

impl<T: ICommentRepository> CommentAppService<T> {
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    pub async fn create(&self, payload: NewComment) -> anyhow::Result<Comment> {
        self.repository.create(payload).await
    }

    pub async fn find(&self, id: i32) -> anyhow::Result<Comment> {
        self.repository.find(id).await
    }

    // TODO Bad approach because it's not scalable
    pub async fn find_by_article_id(&self, article_id: i32) -> anyhow::Result<Vec<Comment>> {
        self.repository.find_by_article_id(article_id).await
    }

    pub async fn update(&self, id: i32, payload: UpdateComment) -> anyhow::Result<Comment> {
        self.repository.update(id, payload).await
    }

    pub async fn delete(&self, id: i32) -> anyhow::Result<()> {
        self.repository.delete(id).await
    }
}
