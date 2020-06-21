# Alica Messages Transaction Processor

Transaction Processor module intended for use within Hyperledger Sawtooth network.
Used for logging task allocation messages, sent by ALCIA agents.

## Payload Structure

The transaction processor accepts payloads in the following form: 

```
agentID|messageType|message|timestamp
```

- `agentID` refers to the sender by its unique identifier contained in the sent message.
- `messageType` identifyies the type of the sent message.
- `message` contains the sent message of type `messagetype`
- `timestamp` contains the Unix timestamp of reception of the sent message by a monitor.

## Address Structure

Hyperledger Sawtooth networks use 70 bytes for addressing transactions.
The addresses in this case are calculated from a combination of the transaction namespace `alica_messages` and data contained in the payload.
The calculation works as follows

```Rust
hex(sha512("alica_messages"))[0..6]
    + hex(sha512(agentID + messageType + timestamp))[0..64]
```

The first 6 bytes of the 70 byte Sawtooth addresses consist of the hash of the transaction namespace.
The remaining 64 bytes are drawn out of a unique combination of the payload components.

## State Rules

`WHEN <address> exists THEN do not update`

`WHEN <address> exists not THEN create new entry <address, payload>`

`WHEN <address> exists multiple times THEN inconsistent state error`