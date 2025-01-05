use async_trait::async_trait;
use sqlx::{types::Uuid, PgPool, Row};
use sqlx::postgres::PgRow;
use time::PrimitiveDateTime;
use crate::models::{postgres_error_codes, Answer, AnswerDetail, DBError};

#[async_trait]
pub trait AnswersDao {
    async fn create_answer(&self, answer: Answer) -> Result<AnswerDetail, DBError>;
    async fn delete_answer(&self, answer_uuid: String) -> Result<(), DBError>;
    async fn get_answers(&self, question_uuid: String) -> Result<Vec<AnswerDetail>, DBError>;
}

pub struct AnswersDaoImpl {
    db: PgPool,
}

impl AnswersDaoImpl {
    pub fn new(db: PgPool) -> Self {
        AnswersDaoImpl {
            db
        }
    }
}

#[async_trait]
impl AnswersDao for AnswersDaoImpl {
    async fn create_answer(&self, answer: Answer) -> Result<AnswerDetail, DBError> {
        const CREATE_ANSWER_QUERY: &str = "
                INSERT INTO answers ( question_uuid, content )
                VALUES ( $1, $2 )
                RETURNING answer_uuid, question_uuid, content, created_at
            ";

        let uuid = Uuid::parse_str(&answer.question_uuid).map_err(|e| {
            DBError::InvalidUUID(format!("Invalid UUID format for '{}': {}", answer.question_uuid, e))
        })?;

        let result: AnswerDetail = sqlx::query(CREATE_ANSWER_QUERY)
            .bind(&uuid)
            .bind(&answer.content)
            .try_map(|row: PgRow| {
                Ok(AnswerDetail {
                    answer_uuid: row.try_get::<Uuid, _>("answer_uuid")?.to_string(),
                    question_uuid: row.try_get::<Uuid, _>("question_uuid")?.to_string(),
                    content: row.try_get("content")?,
                    created_at: row.try_get::<PrimitiveDateTime, _>("created_at")?.to_string()
                })
            })
            .fetch_one(&self.db)
            .await
            .map_err(|e: sqlx::Error| match e {
                sqlx::Error::Database(db_err) => {
                    if let Some(code) = db_err.code() {
                        if code.eq(postgres_error_codes::FOREIGN_KEY_VIOLATION){
                            return DBError::InvalidUUID(format!(
                                "Invalid question UUID: {}",
                                &answer.question_uuid
                            ));
                        }
                    }
                    DBError::Other(Box::new(db_err))
                }
                e => DBError::Other(Box::new(e)),
            })?;

        Ok(result)
    }

    async fn delete_answer(&self, answer_uuid: String) -> Result<(), DBError> {
        const DELETE_ANSWER_QUERY: &str = "DELETE FROM answers WHERE answer_uuid = $1";

        let uuid = Uuid::parse_str(&answer_uuid).map_err(|e| {
            DBError::InvalidUUID(format!("Invalid UUID format for '{}': {}", answer_uuid, e))
        })?;

        sqlx::query(DELETE_ANSWER_QUERY)
            .bind(&uuid)
            .execute(&self.db)
            .await
            .map_err(|e| DBError::Other(Box::new(e)))?;

        Ok(())
    }

    async fn get_answers(&self, question_uuid: String) -> Result<Vec<AnswerDetail>, DBError> {
        const GET_ALL_ANSWERS_QUERY: &str = "SELECT * FROM answers WHERE question_uuid = $1";

        let uuid = Uuid::parse_str(&question_uuid).map_err(|e| {
            DBError::InvalidUUID(format!("Invalid UUID format for '{}': {}", question_uuid, e))
        })?;

        let answers: Vec<AnswerDetail> = sqlx::query(GET_ALL_ANSWERS_QUERY)
            .bind(&uuid)
            .try_map(|row: PgRow| {
                Ok(AnswerDetail {
                    question_uuid: row.try_get::<Uuid, _>("question_uuid")?.to_string(),
                    answer_uuid: row.try_get::<Uuid, _>("answer_uuid")?.to_string(),
                    content: row.try_get("content")?,
                    created_at: row.try_get::<PrimitiveDateTime, _>("created_at")?.to_string()
                })
            })
            .fetch_all(&self.db)
            .await
            .map_err(|e| DBError::Other(Box::new(e)))?;

        Ok(answers)
    }
}