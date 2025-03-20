# bee-dee-lambda

### Overview

This is a GroupMe birthday bot that runs on two AWS Lambda functions, `bd_checker` and `bd_responder`, along with an associated DynamoDB instance for storing the birthdays. The name "Bee-Dee" is a playful take on the phrase "b-day".

`bd_checker` is a simple function that's connected to an EventBridge Scheduler to run every morning; it queries the DynamoDB instance for any birthdays that day, and sends a congratulatory message to the GroupMe group for each one.

`bd_responder` is slightly more complicated, as it receives message callbacks from the GroupMe group, and has to validate and parse those messages to check if it needs to respond. It begins by checking the user agent of the incoming event to check that it's from GroupMe's BotNotifier agent (to filter out anyone just trawling AWS function URLs), and then checks that the sender of the message is a user (to prevent the bot from responding to itself or any other bots, mostly). Finally, it parses the message for the starting command syntax (`!bd`, case insensitive), and then for either of its command phrase (`hello/hi` and `me`, case insensitive), to which it responds appropriately. If the starting command syntax is found but not a valid command phrase, it responds with an error message.

`shared` is the final crate in this workspace, and it's a library crate that contains logic that is shared between the two functions: querying the DynamoDB instance, sending GroupMe messages, etc.

### Motivation

I made an earlier version of this bot in Python that ran on a Raspberry Pi, but I decided to use the project as a good starting point for getting familiar with AWS, and since I prefer Rust to Python, I rewrote it in Rust for the fun of it.

As for why I'm open-sourcing my work, the AWS SDKs aren't very well documented at the time of writing this, so I had to figure out a lot by trial and error. And, when you have to re-build and re-upload for every single trial, it takes a long time to figure out little issues. I hope that this repository can be a learning resource to help others looking to tackle similar projects.

## Building from source

### Prerequisites

* A 2024 edition of the **Rust programming language**, which is versions `1.85.0` and above. You can download Rust from [the Rust website](https://www.rust-lang.org/).

* **Cargo Lambda**, a CLI tool developed by Amazon that integrates with the Rust package manager (`cargo`) to build and deploy Lambda functions. You can find download instructions on [the Cargo Lambda website](https://www.cargo-lambda.info/).


### Building

1. Clone this repository:
    ```bash
    git clone https://github.com/chris-t-jansen/bee-dee-lambda.git
    ```

2. Navigate to the repository folder:
    ```bash
    cd bee-dee-lambda
    ```

3. Add `secrets.rs` to `/shared/src/`. This file contains the necessary secrets to run the bot, such as the ARN for the DynamoDB instance. Secrets should be declared as public `&str` constants (e.g. `pub const MY_SECRET: &str = "my_very_special_secret"`).

4. Run the build command to build the functions:
    ```bash
    cargo lambda build --release
    ```

> [!IMPORTANT]  
> If you're deploying to a Lambda function that is setup to use the `arm64` architecture instead of the `x86_64` architecture, you'll need to append the `--arm64` flag to the above build command.

That should build both binary functions (`bd_checker` and `bd_responder`) to their own folders in the `/target/lambda/` directory. Both built binaries should be named `bootstrap`, which they need to be for the Lambda runtime to detect and run them properly.


## Deploying to AWS Lambda

### Prerequisites

* **AWS Lambda functions** with IAM permissions to read from DynamoDB (for querying birthdays) and write logs to CloudWatch (for logging information/errors).
    * The responder function will need a public function URL to use as the callback URL for the GroupMe bot settings (see below).
    * The checker function will need to be connected to an EventBridge Scheduler to run it on a schedule.

* A **DynamoDB instance** containing the birthdays. The items should all have the following attributes:
    * `user_id`: a `Number` of the user's GroupMe user ID. You can find this by sending an HTTP `GET` request as detailed in [this GroupMe API documentation](https://dev.groupme.com/docs/v3#messages_index).
    * `fullname`: a `String` of the user's name as the bot will print it in birthday messages (e.g. for "John D. Smith", it'll print "It's John D. Smith's birthday today!").
    * `month_num`: a `Number` of the person's birth month (e.g. `4` for April). Should be within the range of `1 - 12`.
    * `day_num`: a `Number` of the person's calendar birth date (e.g. `20` if they were born on the 20th of the month). Should be within the range of `1 - 31`.

    The `user_id` field should be the partition key with no sort key, and you'll need to configure a global secondary index (GSI) named `month-day-index` with `month_num` as the partition key and `day_num` as the sort key so that the bot can query birthdays based on the current date.

* A **GroupMe bot**, which you can create on [this page of the GroupMe Developers website](https://dev.groupme.com/bots). You'll need the bot ID as one of the secrets in your `secrets.rs` file (see above), and you'll need to set the callback URL to the public function URL of the responder function (see above).


### Deploying

> [!NOTE]
> Theoretically, there's a way to deploy the Lambda functions by running `cargo lambda deploy` ([see here](https://www.cargo-lambda.info/guide/getting-started.html#step-6-deploy-the-function-on-aws-lambda)). However, I don't have the AWS CLI installed, so I do it the manual way of uploading `.zip` files.

1. After building the binaries (see above), compress the binaries to individual zip files:

    ```bash
    zip target/lambda/bd_responder/bootstrap.zip target/lambda/bd_responder/bootstrap
    zip target/lambda/bd_checker/bootstrap.zip target/lambda/bd_checker/bootstrap
    ```

2. Modify the permissions of the zip files so that they're executable by the Lambda runtime on AWS:

    ```bash
    chmod a+x target/lambda/bd_responder/bootstrap.zip
    chmod a+x target/lambda/bd_checker/bootstrap.zip
    ```

3. Navigate to the page for your Lambda function in the AWS Management Console.

4. Under the `Code` tab in the `Code Source` box, click on `Upload from` and choose `.zip file`.

5. Select the `bootstrap.zip` file in the build directory for that function (e.g. `/target/lambda/bd_responder/bootstrap.zip` for the responder Lambda function).

6. Upload the zip file and wait for the function to finish updating.

7. Repeat steps 3 - 6 for the other Lambda function.

You should now have both Lambda functions uploaded and automatically running smoothly. If things don't appear to be working, check the latest log streams in CloudWatch. If you can't figure it out, feel free to open an issue.


## License
Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
