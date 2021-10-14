Vote
===
## Features

- Provides a simple voting system in blockchain

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
      "status": "OPEN",
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
  }
  ```

### `get_config`

- Returns a subset of vote data to create the voting paper

  ```json
  args: {}
  ```

### `add_vote`

- Checks if the caller has already been voted
- Checks if the voter id is the transaction submitter
- Checks if the polling station status is "OPEN"
- Checks the vote consistency
- Adds the user vote to the option choose in the voting results
- Burns the voter id to forbid a second votation

  ```json
  args: {
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
      "owner": SUBMITTER_ACCOUNT_ID,
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
