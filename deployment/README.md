# Currently a note space for deployments

## Viewing logs

SSH into machine, use `$ journalctl -u rrmeals.service`.

Should ship them somewhere at some point in time but :shrug: .

## AWS access bits

The Lightsail instance has keys for a role in the main AWS account that has access to a specific DynamoDB table. The source IP address is the static IP of the Lightsail instance. No other access is allowed for that user.

This was done because STS assume-role wasn't playing ball.
