# Coding Rules (FOLLOW STRICTLY)

Aim for best DX;
LLMs and AI agents should be able to run and debug any problem very easily in production;
Maintain all constants and types in central manner in the code, easy to edit and maintain; do not hardcode paths or numbers;
Do NOT over-engineer; Make the code extensible but light and easy to maintain;
Avoid buffer interpretation, memory copies or memory allocations; aim for zero-copy pipelines;
Avoid large monolith files (> 400LOC); split into smaller files; aim for highly reusable functions < 50LOC;
Must have High type-safety with Generics. Avoid making everything as "string", or "number" type, especially the domain objects; use valid enums, types (e.g. FilePath type, FileSize type - instead of string, number);