Vote
===
## Features

- Provides a voting system in blockchain
- The anonymity of the vote is guaranteed by the use of blindly signed tokens


## Methods

### `init`

- Initializes the polling station and opens the votation
  ```json
  args: {
      "title": {
          "en": "President Election and Spring Break lunch proposal"
      },
      "description": {
          "en": "Vote to elect the President and to choose the spring break "
      },
      "start": 1619517600,
      "end": 1619776800,
      "status": "OPEN",
      "anonymous": true,
      "owner": SUBMITTER_ACCOUNT_ID,
      "rules": {
          "min": 1,
          "max": 2
      },
      "questions": [
          {
              "id": "1",
              "question": "Choose your President",
              "rules": {
                  "min": 1,
                  "max": 1
              },
              "options": [
                  {
                      "id": "1",
                      "question": "html",     // Needed by the client side app
                      "value": "John Doe"
                  },
                  ...
              ]
          },
          ...
      ],
      "polling_stations": [
          {
              "id": "W1",
              "uri": "URI1",
              "salt": bytes,
              "pk_rsa": {
                  "e": bytes,
                  "n": bytes,    // (e.g. 256 bytes for 2048 bit RSA key)
              }
          },
          ...
      ],
  }
  ```

### `get_config`

- Returns a subset of vote data to create the voting paper

  ```json
  args: {}
  ```

### `add_vote`

- Checks if the token is valid
- Checks if the token has not already been burned
- Checks if the voter id "belongs" to the current polling station
- Checks if the voter id is the transaction submitter
- Checks if the polling station status is "OPEN"
- Checks the vote consistency
- Adds the user vote to the option choose in the voting results
- Burns the voter token to forbid a second votation with the same token

  ```json
  args: {
      "token": bytes,
      "answers":[
          {
              "id": string,
              "values": ["2",..,"X"]
          },
          ...
      ]
  }
  ```

### `get_result`

- The submitter must be the `Owner`
- Closes the vote (set the `status` to "CLOSED")
- Returns the `votes` field

  ```json
  args: {}
  ```

## Vote Smart Contract fields

- _config_ field:

  ```json
  "config": {
      "title": {
          "en": "President Election and Spring Break lunch proposal"
      },
      "description": {
          "en": "Vote to elect the President and to choose the spring break "
      },
      "anonymous": true,
      "owner": SUBMITTER_ACCOUNT_ID,
      "start": 1619517600,
      "end": 1619776800,
      "status": "OPEN",
      "rules": {
          "min": 1,
          "max": 2
      },
      "questions": [
          {
              "id": "1",
              "question": "Choose your President",
              "rules": {
                  "min": 1,
                  "max": 1
              },
              "options": [
                  {
                      "id": "1",
                      "question": "html",     // Needed by the client side app
                      "value": "John Doe"
                  },
                  ...
              ]
          },
          ...
      ],
      "polling_stations": [
          {
              "id": "W1",
              "uri": "URI1",
              "salt": bytes,
              "pk_rsa": {
                  "e": bytes,
                  "n": bytes,    // (e.g. 256 bytes for 2048 bit RSA key)
              }
          },
          ...
      ],
  }
  ```

- _votes_ field:

  ```json
  "votes": [
  {
      "id" : "1",
      "result": [
          {
              "id": "1",
              "votes": 1000
          },
          ...
          {
              "id": "N",
              "votes": 30
          }
      ],
  },
  ...
  ]
  ```

