# 4RYA

## New data structures

### Certificate

Used by a ```certifier``` to certify a set of target's data.

```json
[
    // data
    [
        // target
        'account id',

        // fields
        [ 'email', 'name', 'sex', 'surname', 'tel' ],

        // salt
        <Bytes b2 35 0c 6f 94 69 64 6d b4...>,

        // root
        <Bytes d3 1e 76 0a 92 13 89 f2...>,

        // certifier
        [
            'ecdsa',
            'secp384r1',
            <Bytes 04 1b 16 d8 d0 4f de cb...>
        ]
    ],

    // signature
    <Bytes 40 b6 9b 57 b2 00 19...>,

    // multiproof
    [
        <Bytes 9d 67 a0 14 9e...>,
        <Bytes cb d5 67 ca 11...>,
        <Bytes ca 8a bc df 2e...>
    ]
]
```

- ```target``` - entity whose data are being certified.
- ```fields``` - names of all fields certified by this certificate.
- ```salt``` - random bytes added to data during merkle tree creation.
- ```root``` - merkle tree root.
- ```certifier``` - public key of the certifying authority.
- ```signature``` - data digital signature.
- ```multiproof``` - additional (not signed) data needed to verify a data set against certificate if the aforementioned data set isn't complete.

### Delegation

Used by ```delegator``` to grant some capabilities for the delegate to operate on target

```json
[
    [
        // delegate
        'QmANv...',

        // delegator
        [
            'ecdsa',
            'secp384r1',
            <Bytes 04 d9 bc cf...>
        ],

        // network
        'skynet',

        // target
        'Qme1Z...',

        // expiration
        1638542148,

        // capabilities
        {
            '*': true,
            secretMethod: false
        }
    ],

    // signature
    <Bytes 0a a6 22 75 f8 01... >
]
```

- ```delegate``` - entity (account) being granted permissions.
- ```delegator``` - entity who grants permissions.
- ```network``` - network name.
- ```target``` - entity (account) on which delegate will be operating.
- ```expiration``` - timestamp representing time until which this delegation is valid. **Currently has no effect**.
- ```capabilities``` - actions which delegate can perform on target. In the example above delegate can do anything except for the "secretMethod".
- ```signature``` - data digital signature.

## Methods

&nbsp;

- ### `init`

    args:

    ```json
    {
        crypto: 'accountId',
    }
    ```

  - crypto - ```string```, account id to which a ```crypto``` smart contract is associated. Is used for merkle tree multiproof verification.

&nbsp;

- ### `set_profile_data`

    If a key is already present in saved data, then it's value gets overwritten.

    All saved data can be found under `<tx_signer_acc_id>:profile_data` on Arya account.

    An entity can manage only his own profile data.

    args:

    ```json
    {
        key1: 'value1',
        key2: 'value2',
        ...
    }
    ```

  - Only string values accepted.
  - Any string can be used as key.

&nbsp;

- ### `remove_profile_data`

    args:

    ```json
    [
        'key1',
        'key2',
        ...
    ]
    ```

  - list of all keys to delete. '*' can be used to remove all data. Bear in mind that all data can still be found inside previously submitted ```set_profile_data``` transactions.

&nbsp;

- ### `set_certificate`

    An entity can manage all of it's own certificates.

    An entity A can manage all of B's certificates of which A is the certifier.

    A list of target's certificates can be found in `<target>:certificates:list` entry of the Arya account.

    Every certificate's data can be found in `<target>:certificates:<certifier>:<key>` entry of the Arya account.

    If a certificate is already present under the same key, it gets overwritten.

    args:

    ```json
    {
        key: 'mykey',
        certificate: <Bytes 0f c5 bd aa f5...>
    }
    ```

  - ```key``` - string, a key to differentiate between various certificate issued from the same authority for the same target.
  - ```certificate``` - actual certificate bytes.

&nbsp;

- ### `remove_certificate`

    Removes one or more certificates from arya. Bear in mind that those certificates can still be found inside previously submitted ```set_certificate``` transactions.

    An entity can manage all of it's own certificates.

    An entity A can manage all of B's certificates of which A is the certifier.

    args:

    ```json
    {
        target: 'accountId',
        certifier: 'accountId',
        keys: ['*'],
    }
    ```

  - target - string, target of the certificate you want to remove.
  - certifier - string, issuer of the certificate you want to remove.
  - keys - one or more keys of the certificate you want to remove. '*' can be used to remove them all.

&nbsp;

- ### `fields_certified`

    Checks whether a field (or a set of fields) is certified by a certain certifier for a certail target.

    args:

    ```json
    {
        target: "<accountId>",
        certifier: "<accountId>";
        key: "<certificateKey>";
        fields: ["fieldName1", "fieldName2"];
    } 
    ```

  - target - string, target, which certificates you want arya to check for the presence of needed fields.
  - certifier - string, issuer of the certificates you want arya to check for the presence of needed fields.
  - key - string, **can be omitted**. If you want to check only a certificate with a specific key, put it here.
  - fields - array of strings, name of the field you want to check certificates for.

    If the ```key``` member is omitted, then arya will check all ```target```'s available certificates issued by ```certifier```.
    &nbsp;
    For example if we have 2 certificates:
    &nbsp;
    certificate key: "main"

    ```json
    {
        target: "userAcc",
        certifier "certifAcc",
        fields: ["name", "surname"]
    }
    ```

    &nbsp;
    certificate key: "secondary"

    ```json
    {
        target: "userAcc",
        certifier "certifAcc",
        fields: ["email", "address"]
    }
    ```

    A call with those args will return ```false```:

    ```json
    {
        target: "userAcc",
        certifier: "certifAcc";
        key: "main";
        fields: ["name", "email"];
    } 
    ```

    While a call with args below will return ```true```:

    ```json
    {
        target: "userAcc",
        certifier: "certifAcc";
        fields: ["name", "email"];
    } 
    ```

&nbsp;

- ### `verify_data`

    Verifies a set of data against a certificate.

    Can be performed by anyone.

    If data set is incomplete relatively to the certificate 'fields' and no multiproof is provided, arya tries to get missing data from target's saved profile data (if there are any).

    > **WARNING**: Another variant of this call is available. Continue reading below.

    args:

    ```json
    {
        target: '<targetAccount>',
        certifier: '<certifierAccount>',
        key: '<certificateKey>',
        data: {
            'key1': 'value1',
            'key2': 'value2'
        },
        multiproof: [
            <Bytes>,
            <Bytes>,
            ...
        ],
    }
    ```

  - target - string, target of the certificate against which you want to verify data.
  - certifier - string, issuer of the certificates you want arya to check for the presence of needed fields.
  - data - string to string map, Set of data which have to be verified.
  - multiroof - array of byte arrays, additional data needed if data set is incomplete and no complementary data is saved in target's profile.

&nbsp;

- ### `verify_data (#2)`

    Verifies a set of data against a certificate (passed with args, not saved).

    Can be performed by anyone.

    Use this variant of the previous call to check a set of data against a provided certificate without saving certificate itself in arya. If data set is incomplete relatively to the certificate 'fields' and no multiproof is provided, arya tries to get missing data from target's saved profile data (if there are any).

    args:

    ```json
    {
        data: {
            'key1': 'value1',
            'key2': 'value2'
        },
        certificate: <Bytes 0f c5 bd aa f5...>
    }
    ```

  - target - string, target of the certificate against which you want to verify data.
  - ```certificate``` - actual certificate bytes.

    > **WARNING**: While you still can provide all the remaining arguments(target, certifier, multiproof etc...) they will be completely ignored. So if you need to check data against a saved certificate, you MUST NOT put any certificate into call's args.

&nbsp;

- ### `set_delegation`

    An entity can manage all of it's own delegations.

    An entity A can manage all of B's delegations of which A is the delegator.

    A list of target's delegations can be found in `<delegate>:delegations:list` entry of the Arya account.

    Every certificate's data can be found in `<delegate>:delegations:<delegator>:<target>` entry of the Arya account.

    If a certificate is already present under the same key, it gets overwritten.

    args:

    ```json
    {
        delegation: <Bytes>,
    }
    ```

  - delegation - byte array, account id to which a ```crypto``` smart contract is associated. Is used for merkle tree multiproof verification.

&nbsp;

- ### `remove_delegation`

    Removes one or more delegations from arya. Bear in mind that those delegations can still be found inside previously submitted ```set_delegation``` transactions.

    An entity can manage all of it's own delegations.

    An entity A can manage all of B's delegations of which A is the delegator.

    args:

    ```json
    {
        delegate: '<accountId>',
        delegator: '<accountId>',
        targets: ['*'],
    }
    ```

  - delegate - string, delegate of the delegation you want to remove.
  - delegator - string, issuer of the delegation you want to remove.
  - targets - one or more targets of the delegation you want to remove. '*' can be used to remove them all.

&nbsp;

- ### `verify_capability`

    Returns true if delegate has been delegated by delegator do call a method on target

    args:

    ```json
    {
        delegate: '<accountId>',
        delegator: '<accountId>',
        target: '<accountId>',
        method: 'method',
    }
    ```

  - delegate - string, delegate of the delegation you want to verify.
  - delegator - string, issuer of the delegation you want to verify.
  - target - target of the delegation you want to verify.
  - method - method presence of which you want to check the delegation for.
