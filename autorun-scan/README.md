# autorun-scan

Scanning module for Autorun, although you can use this on its own.
Simplest form of scanning using `/proc/self/maps` and `/proc/self/mem`.

Implements a macro to make scanning easier.

## Support

- [x] Linux
- [ ] Windows

## Example

```rs
	let scan_result = autorun_scan::scan(autorun_scan::sig![
		0x48, 0x8b, 0x05, ?, ?, ?, ?, 0x55, 0x48, 0x89, 0xe5, 0x5d, 0x48, 0x8b, 0x00
	])?;
```
