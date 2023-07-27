# Testing Testkit with a Real-World API SPEC

The `realworld.yaml` file contains a Testkit version of the [Real World API SPEC](https://github.com/gothinkster/realworld/blob/main/api/Conduit.postman_collection.json), which allows you to experiment with Testkit and get started quickly.

## Server Setup

Before testing the API SPEC, you need to have a server running to test against. There are various implementations of the `realworld` API SPEC in different languages. To use them as test servers, clone the desired implementation from [here](https://codebase.show/projects/realworld?category=backend), and run it locally.

For the purpose of this guide, we'll walk you through setting up the `Go + Gin` implementation.

### Setting up the Go + Gin Implementation of the Real World API

1. Clone the repository and navigate into the directory:

```shell
git clone https://github.com/gothinkster/golang-gin-realworld-example-app.git
cd golang-gin-realworld-example-app/
```

2. Build the server by running the following commands in the project's root directory:

```shell
go build ./...
```

3. Start the server by executing the generated executable:

```shell
./golang-gin-realworld-example-app
```

Once the server is running, you need to set up the environment variables. You have two options: either set them up in a `.env` file or directly in the `realworld.yaml` Testkit file.

The example test file expects the following environment variables:

- `APIURL`
- `PASSWORD`
- `USERNAME`
- `EMAIL`

To set these environment variables in a `.env` file, create the file and add the necessary values:

```shell
APIURL=https://api.example.com
EMAIL=user@example.com
PASSWORD=mysecretpassword
USERNAME=myusername
```

Now, you are ready to run the test. Execute the following command:

```shell
cargo run -- --file ./Examples/realworld.yaml
```

This will run the Testkit test using the specified `realworld.yaml` file against your locally running Go + Gin implementation of the Real World API. Enjoy exploring and testing with Testkit!
