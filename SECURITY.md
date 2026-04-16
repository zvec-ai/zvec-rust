# Security Policy

## Supported Versions

The following versions of zvec-rust are currently supported with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.3.x   | :white_check_mark: |
| < 0.3   | :x:                |

We recommend always using the latest stable version to benefit from the most recent security patches and improvements.

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you discover a security vulnerability in zvec-rust, please report it responsibly.

### How to Report

You have two options for reporting security vulnerabilities:

#### Option 1: GitHub Security Advisories (Preferred)

1. Go to the [Security Advisories](https://github.com/sunhailin-Leo/zvec-rust/security/advisories) page
2. Click "New draft security advisory"
3. Fill in the details of the vulnerability
4. Submit the advisory

This method allows for private discussion and coordinated disclosure.

#### Option 2: Email

Send an email to [sunhailin.shl@antgroup.com](mailto:sunhailin.shl@antgroup.com) with the following information:

- **Description**: Detailed description of the vulnerability
- **Impact**: Potential impact and affected components
- **Reproduction Steps**: Clear steps to reproduce the issue
- **Suggested Fix**: If you have a suggested fix, please include it
- **Your Contact Information**: For follow-up questions

### What to Include

When reporting a vulnerability, please provide:

- Type of vulnerability (e.g., buffer overflow, use-after-free, data exposure)
- Affected versions
- Proof of concept or exploit code (if available)
- Any potential mitigations

## Response Timeline

We commit to the following response timeline:

- **Initial Response**: Within 48 hours of receiving the report
- **Vulnerability Assessment**: Within 5 business days
- **Fix Development**: Within 14 business days for critical vulnerabilities
- **Public Disclosure**: Coordinated with the reporter, typically within 30 days of fix release

For critical vulnerabilities that pose immediate risk, we may expedite this timeline.

## Disclosure Policy

We follow a coordinated disclosure process:

1. **Private Discussion**: The vulnerability is discussed privately between the maintainers and the reporter
2. **Fix Development**: A fix is developed and tested
3. **Release**: A new version is released with the fix
4. **Public Disclosure**: Details of the vulnerability are disclosed publicly after users have had time to update

### Embargo Period

We typically maintain a 30-day embargo period between releasing a fix and publicly disclosing vulnerability details. This gives users time to update their installations.

### Credit

We believe in giving credit where it's due. Security researchers who report vulnerabilities will be credited in the security advisory and release notes, unless they prefer to remain anonymous.

## Security Best Practices for Users

To maintain security when using zvec-rust:

1. **Keep Updated**: Always use the latest stable version
2. **Review Dependencies**: Regularly audit your dependencies for known vulnerabilities
3. **Input Validation**: Validate and sanitize all input before passing to the library
4. **Resource Limits**: Configure appropriate memory and thread limits for your use case
5. **Error Handling**: Properly handle all errors returned by the library

## Scope

This security policy covers:

- The `zvec` crate (safe Rust wrapper)
- The `zvec-sys` crate (FFI bindings)
- Build scripts and configuration
- Official examples and documentation

Out of scope:

- Vulnerabilities in the underlying zvec C library (report to [alibaba/zvec](https://github.com/alibaba/zvec))
- Vulnerabilities in third-party dependencies
- Issues in development-only dependencies

## Contact

For security-related questions or concerns, please contact:

- **Email**: [sunhailin.shl@antgroup.com](mailto:sunhailin.shl@antgroup.com)
- **GitHub**: [Security Advisories](https://github.com/sunhailin-Leo/zvec-rust/security/advisories)

Thank you for helping keep zvec-rust secure!
