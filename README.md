# Readme

## Introduction

Trying to build a basic graphQL server using rust(wrap and juniper crates) for implementing Zeppelin webserver

## Dependencies

* [Rust 2018 edition](https://www.rust-lang.org/tools/install)
* [Mongodb 4.xx](https://docs.mongodb.com/manual/installation/#mongodb-community-edition-installation-tutorials)

## How to run

* Install dependencies(see the list above)
* Run `cd db_dump; ./init.sh` to create a couple of database entries
* Clone this repo
* Run mongodb in default port
* Run `cargo run` in root directory
* Access graphQL kitchensink at "localhost:3030"

## Reference

* http://alex.amiran.it/post/2018-08-16-rust-graphql-webserver-with-warp-juniper-and-mongodb.html
* https://graphql-rust.github.io/quickstart.html
