Below rules and CONSTRAINTS are vital for good quality code, and non-negotiable:

 - keep files small (<400 LOC), and functions < 50 LOC; Split monoliths into smaller related units;
 - design and plan first, implement later; 
   - identify all the core systems, interfaces, schemas and API contracts first; 
   - Use strong typing for domain models;
   - always design for future extensibility;
   - aim for industry standard benchmark performance;
   - keep all constants in the code in one place (easy to maintain) rather than spreading across;
   - aim for zero-copy memory buffers; Improve memory usage and allocation patterns; Consider caching and pre-computation;
 - follow TDD; every part of the system should be testable stand-alone programmatically (with real functionality, not mocks);
   - Test game logic and state transitions;
   - Test edge cases and error conditions;
   - Use Bevy's testing utilities and patterns;
   - Make tests maintainable and reliable;
 - enable dynamic tracing, debug-logs for all methods; Dev should be able to pin-point exactly where a bug is just by reviewing the logs and performance metrics;  
 - every module should be reusable in different projects; Coupling should be VERY LOW;
 - Consider ECS best practices
 - Minimize entity lookups and iterations
 - Use modern Bevy best practices and efficient patterns