#!/bin/bash

MONGO_DATABASE="zeppelin"
MONGODUMP_PATH="./dump/zeppelin"

mongorestore -d $MONGO_DATABASE $MONGODUMP_PATH
