{{/*
Expand the name of the chart.
*/}}
{{- define "perfect-group-allocation.name" -}}
{{- default .Release.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}
