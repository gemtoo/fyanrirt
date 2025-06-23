# Fyanrirt SMPP client
## Why this exists?
This tool is used to test SMPP servers. What `Fyanrirt` does is bind a transceiver, send SMS, print received status objects to console, then gracefully close SMPP channel.
## How to use it?
```
fyanrirt --smsc-name 'ExampleSMSC' --endpoint 'smpp.gemtoo.dev:2775' --system-id 'yourid' --password 'exampl' --system-type 'exampl' send-sms --src 'Alpha Name' --dst '+19478136680' --content 'test content'
```