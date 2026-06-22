# SOSAL GENERAL PUBLIC LICENSE (SosalGPL)

**Version 2.0**

### Preamble

This License (hereinafter referred to as the "License") is a legally binding agreement concluded between the Copyright Holder (hereinafter referred to as the "Licensor") and any individual or legal entity (hereinafter referred to as the "Licensee") that exercises any rights to use, modify, or distribute the software provided under the terms of this License.

The purpose of this License is to establish strict architectural standards designed to ensure the end-to-end security, reliability, and verifiability of derivative software products created based on the Source Code of the Program.

---

### Section 1. Definitions

* **"Licensor"** means the individual or legal entity that holds the exclusive rights to the Program.
* **"Licensee"** means any individual or legal entity that accepts the terms of this License through concurrent or concluent acts.
* **"Program"** means the original software suite (including source code, libraries, accompanying text files, and specifications) distributed under the terms of this License.
* **"Modified Work"** means any derivative work (including forks, adaptations, modules) created by modifying the Program or integrating it into other software suites in accordance with the rules of Section 3 of this License.
* **"Source Code"** means the preferred form of the work for making modifications to it, consisting of human-readable text in programming languages necessary for compiling and building the Program or Modified Work.
* **"Rust Language"** means the compiled, statically typed general-purpose programming language specified and developed by the Rust Foundation.

---

### Section 2. Acceptance of Terms and Grant of Rights

**2.1. Implied Acceptance.**
Any act of copying, compiling, running, modifying, or distributing the Program constitutes full and unconditional acceptance of all sections and terms of this License by the Licensee. If the Licensee does not agree to these terms, they must immediately cease any use of the Program.

**2.2. Scope of Granted Rights.**
The Licensee is granted a non-exclusive, royalty-free, worldwide, perpetual right to use the unmodified Program for any lawful purpose without restrictions regarding fields of endeavor or hardware architectures.

---

### Section 3. Architectural Requirements and Tool Specifications (The 50%+ Criterion)

**3.1. Mandatory Quota.**
The Licensee has the right to create and distribute Modified Works exclusively on the condition that **not less than 50% (fifty percent) of the total Source Code of the Modified Work is written in the Rust programming language**.

**3.2. Tool Specification and Counting Methodology.**
The calculation of Source Lines of Code (SLOC) must be performed using **the `cloc` (Count Lines of Code) utility version 1.96 or higher** or **the `tokei` utility version 12.1.2 or higher**. The following rules apply during the analysis:

* The following are strictly excluded from the calculation: blank lines, comment lines, documentation files (`.md`, `.txt`), configuration files, and build manifests (`Cargo.toml`, `Makefile`, `CMakeLists.txt`).
* **Automatically Generated Code:** Code created by third-party generators (including Protobuf/gRPC serialization files, `bindgen` bindings) is completely excluded from the total volume of the calculation, unless the templates or source instructions for the generator were written directly by the Licensee in the Rust language.
* **FFI Files (Interface Files):** Header files and binding modules (e.g., `.h` files, C stubs) are included in the calculation and are classified according to their actual programming language.
* **External Dependencies:** Third-party libraries (crates) connected without any modification to their own source code by the Licensee are excluded from the calculation. Only code created or modified directly by the Licensee is taken into account.

**3.3. Scope of Application in Complex Architectures.**

* **Monorepositories (Monorepos):** The 50%+ criterion applies in isolation to each independent target build artifact (target/project) that imports or uses the code of the Program, rather than to the entire monorepository as a whole.
* **Microservices Architecture:** In the case of a microservices architecture, the "Modified Work" is recognized exclusively as that specific microservice (container, isolated application) within which the code of the Program is physically executed or modified. Adjacent microservices interacting with it solely via standard network protocols (REST API, gRPC, message queues) do not fall under the scope of this License and are excluded from the quota calculation.

---

### Section 4. Patent License

**4.1. Grant of Patent Rights.**
Subject to the terms and conditions of this License, each contributor (including the original Licensor) hereby grants to the Licensee a perpetual, worldwide, non-exclusive, no-charge, royalty-free, irrevocable patent license to make, have made, use, offer to sell, sell, import, and otherwise transfer the Program, where such license applies only to those patent claims licensable by such contributor that are necessarily infringed by their contribution(s) alone or by combination of their contribution(s) with the Program.

**4.2. Patent Retaliation Clause.**
If the Licensee institutes patent litigation or legal proceedings against any entity (including a cross-claim or counterclaim in a lawsuit) alleging that the Program or a contribution incorporated within the Program constitutes direct or contributory patent infringement, then any patent and copyright licenses granted to said Licensee under this License for that Program shall terminate automatically as of the date such litigation is filed.

---

### Section 5. Objective Control of Unsafe Code (`unsafe`)

**5.1. Mathematical Limit of Unsafe Usage.**
In order to prevent hidden memory safety vulnerabilities, the use of the `unsafe` keyword, as well as blocks of code, functions, traits, and implementations (`impls`) associated with it, is strictly limited by an objective metric.

* The volume of lines of code located directly inside an `unsafe` context **must not exceed 1.0% (one percent)** of the cumulative volume of Source Lines of Code (SLOC) written in the Rust language within the Modified Work.

**5.2. Instrumental Verification.**
Verification of compliance with the specified limit must be carried out using **the `cargo-geiger` utility version 0.11.0 or higher** (or an equivalent static AST analysis tool approved during the audit process). Exceeding the 1.0% limit constitutes a material breach of this License and is legally equivalent to non-compliance with the Rust language quota.

---

### Section 6. Independent Audit and Dispute Resolution Mechanism

**6.1. Audit Demand Notice.**
In the event that the Licensor has reasonable grounds to believe that the Licensee's Modified Work violates the requirements of Sections 3 or 5 of this License, the Licensor has the right to send a written notice to the Licensee demanding a technological audit.

**6.2. Independent Audit Procedure.**
The Licensee **is not required** to disclose their proprietary or closed source code directly to the Licensor.

* The audit shall be conducted by an independent third-party technical auditor or expert organization (hereinafter referred to as the "Auditor"), possessing recognized competence in the field of IT auditing, chosen by mutual consent of the Parties (or appointed by the Chamber of Commerce and Industry at the place of jurisdiction).
* The Auditor shall sign a Non-Disclosure Agreement (NDA) with the Licensee. The Licensee shall provide the Auditor with the complete source code of the Modified Work exclusively for isolated analysis using the tools specified in Sections 3.2 and 5.2.
* Based on the results of the analysis, the Auditor shall send exclusively a **Compliance Report** to both Parties. This report shall contain only the final statistical indicators (percentage of Rust code, percentage of `unsafe` code) and a conclusion on whether the License is adhered to or breached, without revealing any text of the actual source code to the Licensor.

**6.3. Allocation of Audit Costs.**
The costs of the Auditor's services shall initially be borne by the Licensor. However, if the Compliance Report officially records a violation of the License terms by the Licensee, the Licensee shall be obliged to fully reimburse the Licensor for all documented expenses incurred for the execution of the audit.

**6.4. Cure Period.**
If a violation is confirmed by the Auditor, the Licensee is granted a mandatory cure period of **30 (thirty) calendar days** from the date of receipt of the Compliance Report to bring the Modified Work into full compliance with the criteria of this License. During this period, the rights of the Licensee shall not be suspended.

**6.5. Right to Challenge.**
Either Party has the right to challenge the Auditor's Report within 14 (fourteen) calendar days of its receipt by initiating a re-examination involving three independent certified software experts.

---

### Section 7. Termination

If the Licensee fails to remedy the violations identified by the Auditor within the cure period specified in Section 6.4, or refuses to undergo an independent audit, all rights granted to them under this License shall terminate automatically. Any further use, distribution, or execution of the Program or the Modified Work by the Licensee shall be deemed an infringement of exclusive intellectual property rights and prosecuted under copyright law.

---

### Section 8. Disclaimer of Warranties and Limitation of Liability

**8.1. Disclaimer of Warranties.**
THE PROGRAM IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING, WITHOUT LIMITATION, ANY WARRANTIES OF MERCHANTABILITY OR FITNESS FOR A PARTICULAR PURPOSE.

**8.2. Limitation of Liability.**
UNDER NO CIRCUMSTANCES AND UNDER NO LEGAL THEORY SHALL THE LICENSOR BE LIABLE TO ANY PERSON FOR ANY DIRECT, INDIRECT, SPECIAL, INCIDENTAL, OR CONSEQUENTIAL DAMAGES OF ANY CHARACTER ARISING AS A RESULT OF THE USE OR INABILITY TO USE THE PROGRAM, INCLUDING, WITHOUT LIMITATION, COMPILATION ERRORS, RUNTIME FAILURES, OR MEMORY LEAKS CAUSED BY THE LICENSEE'S VIOLATION OF THE ARCHITECTURAL REQUIREMENTS OF THIS LICENSE.

---

### Section 9. Governing Law and Jurisdiction

**9.1. Governing Law.**
This License, its interpretation, performance, and any disputes arising out of or in connection with it (including non-contractual disputes), shall be governed by and construed in accordance with **the substantive laws of the state/country of the principal place of business (official registration) of the Licensor**, excluding its conflict of law provisions.

**9.2. Jurisdiction and Venue.**
All disputes, controversies, or claims arising out of this License shall be resolved through negotiation. If an agreement cannot be reached within 30 days from the date of sending a formal claim, the dispute shall be submitted to **the competent state court or permanently operating arbitration institution at the place of location (registration) of the Licensor**, whose decision shall be final and binding upon both Parties.