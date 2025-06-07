use crate::{
    model::{api_response::ApiResponse, error_message::ErrorMessage},
    utils::app_error::AppError,
};

pub trait ErrorHandler {
    fn handle_error(
        &self,
        err_message_str: &str,
    ) -> crate::model::api_response::ApiResponse<String>;

    fn log_error(&self);
}

impl ErrorHandler for AppError {
    fn handle_error(
        &self,
        err_message_str: &str,
    ) -> crate::model::api_response::ApiResponse<String> {
        self.log_error();

        let err_msg = ErrorMessage::new(self.error_message_kind(), err_message_str.to_string());

        ApiResponse::new(
            self.status_code(),
            Some(serde_json::to_string(&err_msg).unwrap()),
            None,
        )
    }

    fn log_error(&self) {
        tracing::error!(
            error.kind = %self.error_log_kind(),
            error.message = %self.to_string(),
        );
    }
}

// implement ResultErrorHandler for Result<T, E>
// pub trait ResultErrorHandler<T> {
//     fn handle_app_error(
//         self,
//         user_message: &str,
//     ) -> Result<T, crate::model::api_response::ApiResponse<String>>;

//     fn log_error(self) -> Self;
// }

// impl<T, E> ResultErrorHandler<T> for Result<T, E>
// where
//     E: Into<anyhow::Error>,
// {
//     fn handle_app_error(
//         self,
//         user_message: &str,
//     ) -> Result<T, crate::model::api_response::ApiResponse<String>> {
//         self.map_err(|e| {
//             let anyhow_err = e.into();
//             handle_anyhow_error(anyhow_err, user_message)
//         })
//     }

//     fn log_error(self) -> Self {
//         if let Err(ref e) = self {
//             let anyhow_err: anyhow::Error = e.clone().into();
//             let app_err = AppError::from(anyhow_err);
//             app_err.log_error();
//         }
//         self
//     }
// }

// pub fn handle_anyhow_error(
//     e: anyhow::Error,
//     user_message: &str,
// ) -> crate::model::api_response::ApiResponse<String> {
//     let app_err = AppError::from(e);
//     app_err.handle_error(user_message)
// }

// macro pattern
// #[macro_export]
// macro_rules! handle_result {
//     ($result:expr, $user_message:expr) => {
//         $result.map_err(|e| crate::utils::error_handler::handle_anyhow_error(e, $user_message))
//     };
// }

// #[macro_export]
// macro_rules! handle_error {
//     ($result:expr, $user_message:expr) => {
//         match $result {
//             Ok(value) => Ok(value),
//             Err(e) => {
//                 let app_err = crate::utils::app_error::AppError::from(e);
//                 Err(app_err.handle_error($user_message))
//             }
//         }
//     };
// }

// #[macro_export]
// macro_rules! log_error {
//     ($result:expr) => {
//         if let Err(e) = &$result {
//             let app_err = crate::utils::app_error::AppError::from(e.clone());
//             app_err.log_error();
//         }
//     };
// }
