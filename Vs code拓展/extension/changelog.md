# Changelog

## V 1.1.0 2025/11/26

Smarter Planning, Better Collaboration, and Smoother Conversations

**New Features:**

* **Clarification Loop:** You can now discuss and refine your requirements continuously with AI. This feature lets you collaboratively confirm your plans and provides new interactions, including regenerating, modifying clarification options, skipping clarifications, rolling back changes and resubmit.

<img  width="100%" height="100%"  src="https://cdn.verdent.ai/clddeck_changelog/clarification_loop.gif"> 

* **Plan Rules:** You can adjust how clarifications and plans are presented based on your role and professional background. Choose from role templates or customize content to get professional, personalized plans that match your thinking style and work habits.

<img  width="100%" height="100%"  src="https://cdn.verdent.ai/clddeck_changelog/plan_rules.gif"> 

* **Plan Download:** You can now easily save your plans to your current workspace or local code repository, making it convenient to store and reference your work.

<img  width="100%" height="100%"  src="https://cdn.verdent.ai/clddeck_changelog/plan_download.jpeg"> 

**Experience Improvements:**

* **Conversation Flow Optimization:** You can give feedback on AI-generated content with thumbs up/down and see timestamps for each conversation.

* **Bug Fixes:** We've fixed issues that affected your experience, including login interface optimization, handling of long inputs, black screens when opening history messages, navigation errors, and “thinking” messages that blocked further conversation.



## V 1.0.6 2025/10/29

**Switch, Chat, Explore — Faster, Smoother, Smarter**

**New Features**

* Model Switching: Seamlessly switch between Claude, GPT, and MiniMax-m2 models without restarting your workspace.

* Chat Support: Engage directly with Verdent through an integrated chat interface.

* Explore & Code Reviewer Subagents: Discover new contexts and get automated reviews with specialized subagents.

**Experience Improvements**

* Complete redesign of the Home and Chat interfaces for a more intuitive workflow.

* Performance Optimization: Faster response, reduced latency, and smoother interactions.

* Smaller Package Size for quicker installation and updates.

* Multiple bug fixes for improved stability.

Model switching:

<video  width="100%" height="100%"  src="https://cdn.verdent.ai/CLDDECK_CHANGELOG/files/model_switching.mp4">
</video>

Subagents:

<img  width="100%" height="100%"  src="https://cdn.verdent.ai/clddeck_changelog/subagents.gif">

Chat mode:

<img  width="100%" height="100%"  src="https://cdn.verdent.ai/clddeck_changelog/chat_mode.gif">


## V 1.0.5  2025/10/16

**Faster, Safer, and More Reliable Coding**

This update brings better stability, smarter exception handling, and a smoother user experience, including a new Free Trial mode. Designed to handle your coding tasks more efficiently.

**New Features**

* Browser tool: added copy functionality

* Network retry optimization: increased reliability against short-term network failures

* Files with no diffs can now be opened in concurrent scenarios

* Optimized memory management for smoother performance

**Bug Fixes**

* Fixed virtual scroll errors when switching large messages via the history list

* Fixed data resend issues during Clarify + Plan reconnection after network loss

* Fixed deletion failures of historical messages in directories outside the workspace

* Fixed unprocessed messages under network failure conditions

* De-duplicated global uncaught exception data

* Improved Think Hard display styling

* Clean up old socket instances when switching accounts to ensure tokens are tied to the latest account



## V 1.0.3  2025/10/10

**Smoother Workflow, Safer Git, and Enhanced UX**

We cleaned up some rough edges: more accurate behavior, faster response times, quicker navigation, and safer Git.&#x20;

**Fixes**

* Terminal scrollbar overlay → fixed

* Clear History is now workspace-scoped

* Empty metadata is handled safely

* Add to Verdent path → correct & openable

* git diff excludes .verdent / .git

* Persisted files can be opened immediately

**Manual Mode & Telemetry**

* Rollback triggers on user-rejected actions

* Support for reviewing changes in Manual Mode

* More precise checkpoint-failure instrumentation

* git push confirmation dialog

**UX Polish**

* Virtualized scrolling for History and Messages

* Support for custom themes in VS Code

* Dynamic @ indexes new files; @folder jump

* Auto-retry on transient network failures&#x20;



# V1.0.2 2025/09/25

**Critical Bug Fixes and Improved Reliability**

**Bug Fixes:**

1. Resolved the sign-up issue in registration and login.

2. Fixed the image URL issue in the plugin marketplace.

3. Added support for compressing images larger than 5 MB and improved error message callbacks.

4. Fixed the initial-letter display error when using the Pinyin input method.

5. Fixed the issue where context icons were missing.

6. Disabled browser error message pop-ups.



# V1.0.0 2025/09/23

**Efficient, Transparent, and Intelligent Coding Workflow**

* **Sub-agent Scheduling:** Assign specialized sub-agents to different tasks. The main agent orchestrates them and integrates results, reducing context load.

<img  width="100%" height="100%"  src="https://cdn.verdent.ai/CLDDECK_CHANGELOG/images/image.png">

* **Parallel Task Execution:** Supports breaking down complex requirements into parallelizable subtasks to improve overall development efficiency.

<img  width="100%" height="100%"  src="https://cdn.verdent.ai/CLDDECK_CHANGELOG/images/image-1.png">

* **Plan First, Then Execute:** After receiving a requirement, the system first generates a clear task outline and execution plan. Users can confirm or adjust it before entering the coding phase, reducing unnecessary rework.

<img  width="100%" height="100%"  src="https://cdn.verdent.ai/CLDDECK_CHANGELOG/images/image-2.png">

* **Verify Before Delivery:** Uses fail-fast code checks to provide rapid feedback and ensure correctness before final output.

<img  width="100%" height="100%"  src="https://cdn.verdent.ai/CLDDECK_CHANGELOG/images/image-3.png">

* **Transparent Process with Controllable Details:** Code generation unfolds step by step with interpretable reasoning at each stage. Users can intervene at any time to inspect logic, adjust the workflow, or switch approaches.

<img  width="100%" height="100%"  src="https://cdn.verdent.ai/CLDDECK_CHANGELOG/images/image-4.png">

* **MCP (Model Context Protocol):** Seamlessly integrates external tools and services, supports existing toolchains, and allows custom extensions. Works collaboratively with sub-agents to complete tasks.

<img  width="100%" height="100%"  src="https://cdn.verdent.ai/CLDDECK_CHANGELOG/images/image-5.png">

* **Adaptive Context Compression:** Automatically distills key information and removes redundant history, maintaining "long-term memory + efficient short-term reasoning" to keep large projects running smoothly.

* **Context-Aware Programming:** Continuously tracks the codebase, task goals, and conversation history to avoid forgetting or duplication. It intelligently detects dependencies across code modules to maintain overall consistency during edits.



