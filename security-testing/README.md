# Manual testing guide (for pre-release testing)

https://owasp.org/www-project-web-security-testing-guide/
https://github.com/OWASP/wstg/releases/download/v4.2/wstg-v4.2.pdf

https://owasp.org/www-project-security-culture/
https://github.com/OWASP/security-culture/releases/download/v1.0/OWASP_Security_Culture-1.0.pdf

## OWASP Threat Dragon

https://github.com/OWASP/threat-dragon
https://owasp.org/www-project-threat-dragon/
(download AppImage from releases)

## OWASP Application Security Verification Standard

https://owasp.org/www-project-application-security-verification-standard/
https://github.com/OWASP/ASVS/raw/v4.0.3/4.0/OWASP%20Application%20Security%20Verification%20Standard%204.0.3-en.pdf

# Automated testing guide (for continous testing)

https://owasp.org/www-project-benchmark/

## OWASP Dependency-Track

issues:
https://github.com/DependencyTrack/dependency-track/issues/170

https://owasp.org/www-project-dependency-track/
https://docs.dependencytrack.org/

```
podman run -d -m 8192m -p 8080:8080 --name dependency-track -v dependency-track:/data docker.io/dependencytrack/bundled
```
http://localhost:8080/
username: admin
password: admin

https://crates.io/crates/cargo-cyclonedx
cargo install cargo-cyclonedx
cargo cyclonedx --all # license fails to get extracted from dependencytrack

https://github.com/CycloneDX/cdxgen
podman run --env FETCH_LICENSE=true --rm -v /tmp:/tmp -v $(pwd):/app:rw -t ghcr.io/cyclonedx/cdxgen -r /app -o /app/bom.xml # best solution for now

https://github.com/CycloneDX/cyclonedx-cli
podman run --rm -v $(pwd):/app:rw docker.io/cyclonedx/cyclonedx-cli validate --input-file /app/bom.xml

## Zed Attack Proxy (ZAP)

https://www.zaproxy.org/

This should be a big focus

## https://www.zaproxy.org/docs/automate/



## sqlmap

https://sqlmap.org/

## slowloris

probably no https support
https://github.com/shekyan/slowhttptest/wiki/InstallationAndUsage
podman run --network=host docker.io/shekyan/slowhttptest:latest -u https://h3.selfmade4u.de:8443/ -c 10000 -r 1000 -i 3 -v 1 -x 1000

## metasploit

(isn't there also an automatic mode?)

ulimit -S -n 65536
RUST_LOG=trace DATABASE_URL="sqlite:./sqlite.db?mode=rwc" cargo run --release --bin server

https://github.com/rapid7/metasploit-framework/blob/master//modules/auxiliary/dos/http/slowloris.py

ulimit -S -n 110000
msfconsole
use auxiliary/dos/http/slowloris
show options
set rhost h3.selfmade4u.de
set rport 8443
set ssl true
set sockets 100000
run

we now get to other limits namely local port range
https://stackoverflow.com/questions/410616/increasing-the-maximum-number-of-tcp-ip-connections-in-linux
sysctl net.ipv4.ip_local_port_range="15000 61000"

ip_local_port_range
ip_local_reserved_ports
sudo sysctl -w net.ipv4.ip_local_port_range="1024 65535"

https://blackarch.org/tools.html
https://blackarch.org/dos.html

# https://owasp.org/projects/



# https://github.com/owtf/owtf

https://owtf.github.io/

https://owtf.readthedocs.io/en/develop/

# https://github.com/OWASP/Nettacker

https://github.com/OWASP/Nettacker/wiki/Usage

# https://github.com/owasp-amass/amass

# https://www.securecodebox.io/