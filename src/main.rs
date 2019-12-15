#![allow(non_snake_case)]
#[macro_use]
extern crate juniper;
use bson::{from_bson, oid::ObjectId, Bson, Document};
use juniper::FieldResult;
use mongodb::{coll::Collection, db::ThreadedDatabase, Client, ThreadedClient};
use std::sync::Arc;
use warp::{filters::BoxedFilter, Filter};

#[macro_use(bson, doc)]
extern crate bson;
extern crate mongodb;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[derive(juniper::GraphQLObject, Serialize, Deserialize, Debug)]
#[graphql(description = "Paragraph is a unit of combined code and result")]
pub struct Paragraph {
  pub id: String,
  pub code: String,
  pub result: String,
}

#[derive(juniper::GraphQLObject, Serialize, Deserialize)]
#[graphql(description = "A notebook is a collection of paragraphs")]
pub struct Notebook {
  pub id: String,
  pub name: String,
  pub paragraphs: Vec<String>,
}

pub struct DatabasePool {
  client: Client,
  db_name: String,
}

impl DatabasePool {
  pub fn new<S>(db_name: S) -> DatabasePool
  where
    S: ToString,
  {
    let db_name = db_name.to_string();
    let client = Client::connect("localhost", 27017).expect("Failed to initialize client.");
    DatabasePool { client, db_name }
  }

  pub fn find_notebook(&self, id: &str) -> FieldResult<Notebook> {
    let coll: Collection = self.client.db(&self.db_name).collection("notebooks");
    let cursor: Option<Document> =
      coll.find_one(Some(doc! { "_id": ObjectId::with_string(id)? }), None)?;
    cursor
      .map(|row| Ok(from_bson::<Notebook>(Bson::Document(row))?))
      .unwrap()
  }

  pub fn find_notebooks(&self) -> FieldResult<Vec<Notebook>> {
    let coll: Collection = self.client.db(&self.db_name).collection("notebooks");
    let cursor = coll.find(None, None)?;
    let res: Result<Vec<_>, _> = cursor
      .map(|row| row.and_then(|item| Ok(from_bson::<Notebook>(Bson::Document(item))?)))
      .collect();

    Ok(res?)
  }

  pub fn find_paragraph(&self, id: &str) -> FieldResult<Paragraph> {
    let coll: Collection = self.client.db(&self.db_name).collection("paragraphs");
    let cursor: Option<Document> =
      coll.find_one(Some(doc! { "_id": ObjectId::with_string(id)? }), None)?;
    cursor
      .map(|row| Ok(from_bson::<Paragraph>(Bson::Document(row))?))
      .unwrap()
  }
}

// Now, we create our root Query and Mutation types with resolvers by using the
// object macro.
// Objects can have contexts that allow accessing shared state like a database
// pool.
#[derive(Clone)]
pub struct Context {
  // Use your real database pool here.
  pub db: Arc<DatabasePool>,
}

impl juniper::Context for Context {}

pub struct Query;
#[juniper::object(
    // Here we specify the context type for the object.
    // We need to do this in every type that
    // needs access to the context.
    Context = Context,
)]
impl Query {
  fn apiVersion() -> &str {
    "1.0"
  }

  // query: { notebooks { id, name } }
  fn notebooks(context: &Context) -> FieldResult<Vec<Notebook>>{
    let notebooks = context.db.find_notebooks()?;
    Ok(notebooks)
  }

  // Arguments to resolvers can either be simple types or input objects.
  // To gain access to the context, we specify a argument
  // that is a reference to the Context type.
  // Juniper automatically injects the correct context here.
  fn notebook(context: &Context, id: String) -> FieldResult<Notebook> {
    // Execute a db query.
    // Note the use of `?` to propagate errors.
    let notebook = context.db.find_notebook(&id)?;
    // Return the result.
    Ok(notebook)
  }

  fn paragraph(context: &Context, id: String) -> FieldResult<Paragraph> {
    // Execute a db query.
    // Note the use of `?` to propagate errors.
    let paragraph = context.db.find_paragraph(&id)?;
    // Return the result.
    Ok(paragraph)
  }
}

// Todo: implement
// This is implementation for insertion and stuff
pub struct Mutations;
graphql_object!(Mutations: Context |&self| {
  field create_notebook(&executor, id: String) -> FieldResult<Notebook> {
    let notebook = executor.context().db.find_notebook(&id)?;
    Ok(notebook)
  }
});

pub type Schema = juniper::RootNode<'static, Query, Mutations>;

pub fn make_graphql_filter<Query, Mutation, Context>(
  path: &'static str,
  schema: juniper::RootNode<'static, Query, Mutation>,
  ctx: Context,
) -> BoxedFilter<(impl warp::Reply,)>
where
  Query: juniper::GraphQLType<Context = Context, TypeInfo = ()> + Send + Sync + 'static,
  Context: juniper::Context + Send + Sync + Clone + 'static,
  Mutation: juniper::GraphQLType<Context = Context, TypeInfo = ()> + Send + Sync + 'static,
{
  let schema = Arc::new(schema);

  let context_extractor = warp::any().map(move || -> Context { ctx.clone() });

  let handle_request = move |context: Context,
                             request: juniper::http::GraphQLRequest|
        -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(&request.execute(&schema, &context))
  };

  warp::post2()
    .and(warp::path(path.into()))
    .and(context_extractor)
    .and(warp::body::json())
    .map(handle_request)
    .map(build_response)
    .boxed()
}

fn build_response(response: Result<Vec<u8>, serde_json::Error>) -> warp::http::Response<Vec<u8>> {
  match response {
    Ok(body) => warp::http::Response::builder()
      .header("content-type", "application/json; charset=utf-8")
      .body(body)
      .expect("response is valid"),
    Err(_) => warp::http::Response::builder()
      .status(warp::http::StatusCode::INTERNAL_SERVER_ERROR)
      .body(Vec::new())
      .expect("status code is valid"),
  }
}

pub fn web_index() -> Result<impl warp::Reply, warp::Rejection> {
  Ok(
    warp::http::Response::builder()
      .header("content-type", "text/html; charset=utf-8")
      .body(juniper::graphiql::graphiql_source("/query"))
      .expect("response is valid"),
  )
}

fn main() {
  // creates graph-ql index for localhost:3030
  let gql_index = warp::get2().and(warp::path::end()).and_then(web_index);

  let ctx = Context {
    db: Arc::new(DatabasePool::new("zeppelin")),
  };
  let schema = Schema::new(Query, Mutations);
  // handles a query
  let gql_query = make_graphql_filter("query", schema, ctx);

  let routes = gql_index.or(gql_query);

  warp::serve(routes).run(([127, 0, 0, 1], 3030));
}
