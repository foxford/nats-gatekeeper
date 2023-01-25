{{/*
Expand the name of the chart.
*/}}
{{- define "nats-gatekeeper.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "nats-gatekeeper.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "nats-gatekeeper.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "nats-gatekeeper.labels" -}}
helm.sh/chart: {{ include "nats-gatekeeper.chart" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{ include "nats-gatekeeper.selectorLabels" . }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "nats-gatekeeper.selectorLabels" -}}
app.kubernetes.io/name: {{ include "nats-gatekeeper.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Tenant Service Audience
*/}}
{{- define "nats-gatekeeper.tenantServiceAudience" -}}
{{- $tenant := . -}}
{{- list "svc" $tenant | join "." -}}
{{- end -}}

{{/*
Tenant User Audience
*/}}
{{- define "nats-gatekeeper.tenantUserAudience" -}}
{{- $tenant := . -}}
{{- list "usr" $tenant | join "." -}}
{{- end -}}

{{/*
Tenant Object Audience
*/}}
{{- define "nats-gatekeeper.tenantObjectAudience" -}}
{{- $namespace := index . 0 -}}
{{- $tenant := index . 1 -}}
{{- $env := regexSplit "-" $namespace -1 | first -}}
{{- $devEnv := ""}}
{{- if ne $env "p" }}
{{- $devEnv = regexReplaceAll "(s)(\\d\\d)" $env "staging${2}" }}
{{- $devEnv = regexReplaceAll "(t)(\\d\\d)" $devEnv "testing${2}" }}
{{- end }}
{{- list $devEnv $tenant | compact | join "." }}
{{- end }}

{{/*
Namespace in ingress path.
converts as follows:
- testing01 -> t01
- staging01-classroom-ng -> s01/classroom-foxford
- production-webinar-ng -> webinar-foxford
*/}}
{{- define "nats-gatekeeper.ingressPathNamespace" -}}
{{- $ns_head := regexSplit "-" .Release.Namespace -1 | first }}
{{- $ns_tail := regexSplit "-" .Release.Namespace -1 | rest | join "-" | replace "ng" "foxford" }}
{{- if has $ns_head (list "production" "p") }}
{{- $ns_tail }}
{{- else }}
{{- list (regexReplaceAll "(.)[^\\d]*(.+)" $ns_head "${1}${2}") $ns_tail | compact | join "/" }}
{{- end }}
{{- end }}

{{/*
Ingress path.
*/}}
{{- define "nats-gatekeeper.ingressPath" -}}
{{- list "" (include "nats-gatekeeper.ingressPathNamespace" .) (include "nats-gatekeeper.fullname" .) | join "/" }}
{{- end }}

{{/*
Create volumeMount name from audience and secret name
*/}}
{{- define "nats-gatekeeper.volumeMountName" -}}
{{- $audience := index . 0 -}}
{{- $secret := index . 1 -}}
{{- printf "%s-%s-secret" $audience $secret | replace "." "-" | trunc 63 | trimSuffix "-" }}
{{- end }}
