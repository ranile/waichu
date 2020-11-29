use http_api_problem::HttpApiProblem;
use warp::http::StatusCode;

pub fn from_anyhow(e: anyhow::Error) -> HttpApiProblem {
    let e = match e.downcast::<HttpApiProblem>() {
        Ok(problem) => return problem,
        Err(e) => e,
    };
    HttpApiProblem::new(e.to_string()).set_status(warp::http::StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn problem_to_reply(problem: HttpApiProblem) -> warp::reply::WithStatus<warp::reply::Json> {
    warp::reply::with_status(
        warp::reply::json(&problem),
        problem.status.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
    )
}
