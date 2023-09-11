# Radicle CI

This project is a proof of concept for a CI system for Radicle projects. It is currently in a very early stage of
development and, for the time being, is using [Concourse CI](https://concourse-ci.org/) for scheduling and executing CI
pipelines.

Radicle CI subscribes to node events and is specifically interested in `radicle::node::Event::RefsFetched` events.
It looks for `RefUpdate::Updated` and `RefUpdate::Updated` references related to patches. It will then trigger a
pipeline job for each patch that was updated or created. The pipeline job will clone the repository, checkout the patch,
and execute whatever is declared in the `{project_root_folder}/.concourse/config.yaml` file. This configuration file
contains a description of pipelines and tasks that Concourse CI will execute.

## Installation

### 1. Docker Compose

To install and run Concourse CI, you will first need Docker Compose. Follow the steps documented in the
[official guide](https://docs.docker.com/compose/install/) for your system.

### 2. Concourse CI

Once Docker Compose is installed, proceed with the installation of Concourse CI by following the
[official quick start guide](https://concourse-ci.org/quick-start.html). The installation guide is divided into two
sections.

The first section involves downloading the Docker Compose configuration and executing `docker-compose up` from the
folder where the configuration was stored. This step is necessary as it will download all the required images and spin
up the containers. After this step, Concourse CI will be running, and a UI client will be accessible at
`http://localhost:8080`. The default username and password are both `test`.

The second part is the installation of the fly CLI. While not a strict requirement, it is recommended as it simplifies
interactions and experimentation with the CI.

## Running

Before running Radicle CI, ensure that both the Radicle node and Concourse CI are up and running. Executing `cargo run`
from the root folder of the project is more than enough.

The CLI accepts the following required parameters:

1. `--concourse-url`: This is where Concourse runs. For example, if run locally, it will be `http://localhost:8080`.
2. `--concourse-user`: The username used to authenticate with Concourse.
3. `--concourse-pass`: The password used to authenticate with Concourse.
4. `--radicle-api-url`: This is where the `radicle-httpd` runs. This will be used by Concourse to `git clone` a
   repository.

For the time being, Radicle CI makes one assumptions. The repository that will be cloned to trigger a pipeline job
contains a configuration file located at the following path: `{project_root_folder}/.concourse/config.yaml`.
For example, for the heartwood project, the path would be `heartwood/.concourse/config.yaml`.

## License

Radicle is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.