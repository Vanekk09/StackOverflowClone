use async_trait::async_trait;
use sqlx::{PgPool, Row};
use sqlx::postgres::PgRow;
use sqlx::types::Uuid;
use time::PrimitiveDateTime;
use crate::models::{DBError, Question, QuestionDetail};

#[async_trait]
pub trait QuestionsDao {
    async fn create_question(&self, question: Question) -> Result<QuestionDetail, DBError>;
    async fn delete_question(&self, question_uuid: String) -> Result<(), DBError>;
    async fn get_questions(&self) -> Result<Vec<QuestionDetail>, DBError>;
}

pub struct QuestionsDaoImpl {
    db: PgPool,
}

impl QuestionsDaoImpl {
    pub fn new(db: PgPool) -> Self {
        QuestionsDaoImpl {
            db
        }
    }
}

#[async_trait]
impl QuestionsDao for QuestionsDaoImpl {
    async fn create_question(&self, question: Question) -> Result<QuestionDetail, DBError> {
        const CREATE_QUESTION_QUERY: &str = "
            INSERT INTO questions ( title, description )
            VALUES ( $1, $2 )
            RETURNING question_uuid, title, description, created_at
        ";

        let result: QuestionDetail = sqlx::query(CREATE_QUESTION_QUERY)
            .bind(&question.title)
            .bind(&question.description)
            .try_map(|row: PgRow| {
                Ok(QuestionDetail {
                    question_uuid: row.try_get::<Uuid, _>("question_uuid")?.to_string(),
                    title: row.try_get("title")?,
                    description: row.try_get("description")?,
                    created_at: row.try_get::<PrimitiveDateTime, _>("created_at")?.to_string()
                })
            })
            .fetch_one(&self.db)
            .await
            .map_err(|e| DBError::Other(Box::new(e)))?;

        Ok(result)
    }

    async fn delete_question(&self, question_uuid: String) -> Result<(), DBError> {
        const DELETE_QUESTION_QUERY: &str = "DELETE FROM questions WHERE question_uuid = $1";

        let uuid = Uuid::parse_str(&question_uuid).map_err(|e| {
            DBError::InvalidUUID(format!("Invalid UUID format for '{}': {}", question_uuid, e))
        })?;

        sqlx::query(DELETE_QUESTION_QUERY)
            .bind(uuid)
            .execute(&self.db)
            .await
            .map_err(|e| DBError::Other(Box::new(e)))?;

        Ok(())
    }

    async fn get_questions(&self) -> Result<Vec<QuestionDetail>, DBError> {
        const GET_ALL_QUESTIONS_QUERY: &str = "SELECT * FROM questions";

        let questions: Vec<QuestionDetail> = sqlx::query(GET_ALL_QUESTIONS_QUERY)
            .try_map(|row: PgRow| {
                Ok(QuestionDetail {
                    question_uuid: row.try_get::<Uuid, &str>("question_uuid")?.to_string(),
                    title: row.try_get("title")?,
                    description: row.try_get("description")?,
                    created_at: row.try_get::<PrimitiveDateTime, _>("created_at")?.to_string()
                })
            })
            .fetch_all(&self.db)
            .await
            .map_err(|e| DBError::Other(Box::new(e)))?;

        Ok(questions)
    }
}