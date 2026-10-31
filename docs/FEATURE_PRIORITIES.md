# Feature priorities: evidence and limits
Reviewed 2026-10-31.

## What the evidence supports
Jensen Harris published Word 2003 opt-in command telemetry in 2006: Paste, Save, Copy, Undo and Bold led the ranking and together represented about 32% of command use. Font size was eleventh. Commands included keyboard, toolbar and menu invocations. This is historical evidence about commands in that product, not a contemporary ranking of people or GNOME workflows. Cursor movement and deletion were counted separately.
Source: https://jensenharris.com/blog/no-distaste-for-paste-why-the-ui-part-7

The Document Foundation's 2017 survey received 10,449 submissions, with roughly a third empty. It investigated selected uncommon/legacy features, not the full Writer command set. Its discussion warns about expert self-selection and ambiguous interpretation. It cannot establish a general most-used-functions ranking.
Source: https://design.blog.documentfoundation.org/2017/06/28/results-survey-libreoffice-features/

A CHIIR 2018 study analyzed a month of 2017 Office command-search sessions. Word searches included proofing and printing, but this measures search behavior, not overall feature invocation; suggested queries also affected search volume. It supports investigating discoverability and spelling as task needs, not ranking them above copy/paste.
Source: https://www.microsoft.com/en-us/research/wp-content/uploads/2018/01/search_in_office__chiir2018.pdf

A Writer templates survey conducted in December 2024 and published in January 2025 had 241 respondents. Reported friction included repeated adjustments of margins, fonts and styles; academic documents, resumes, business letters and invoices featured prominently. This concerns a self-selected templates survey, not representative command telemetry. It suggests testing complete document workflows and considering templates later.
Source: https://design.blog.documentfoundation.org/2025/01/13/results-from-a-survey-about-writer-templates/

## Decision for Leaf (inference)
Protect the common editing core first: dependable copy/paste, save/reopen, undo and basic formatting. Measure reliability as well as frequency: recovery is valuable even when rarely invoked.

Columns are a concrete frequent workflow for the project owner and remain a priority regardless of an aggregate ranking. Access is in Paragraph, with explicit selection/section/document scope because storage belongs to document sections.

After columns, candidate task blockers to investigate with testers are printing, spelling/language support, and headers/footers/page numbering. This is a proposed validation list, not a data-backed popularity order. Ask testers which real task they could not finish and request a minimal shareable example. Avoid adding telemetry without a separate decision.

No recent representative public command-use dataset for GNOME document editors was found in this review. Do not turn absence from a published list into evidence of low importance.
