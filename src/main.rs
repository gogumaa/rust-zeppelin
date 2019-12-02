use warp::Filter;

pub fn web_index() -> Result<impl warp::Reply, warp::Rejection> {
  Ok(warp::http::Response::builder()
    .header("content-type", "text/html; charset=utf-8")
    .body(juniper::graphiql::graphiql_source("/query"))
    .expect("response is valid"))
}

fn main() {
  // creates graph-ql index for localhost:3030
  let gql_index = warp::get2()
    .and(warp::path::end())
    .and_then(web_index);

  // handles a query
  // let gql_query = make_graphql_filter("query", schema, ctx);

  let routes = gql_index;
  // let routes = gql_index.or(gql_query);

  warp::serve(routes).run(([127, 0, 0, 1], 3030));
}
