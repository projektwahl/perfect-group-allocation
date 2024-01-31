FROM quay.io/keycloak/keycloak:23.0 as builder

ENV KC_DB=postgres

WORKDIR /opt/keycloak
RUN /opt/keycloak/bin/kc.sh build

FROM quay.io/keycloak/keycloak:23.0
COPY --from=builder /opt/keycloak/ /opt/keycloak/

ENV KC_DB=postgres
ENTRYPOINT ["/opt/keycloak/bin/kc.sh"]