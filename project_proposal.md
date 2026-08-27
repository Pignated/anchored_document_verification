# Nathan Cobb UGR Proposal

## Research topic:

Selective disclosure of documents utilizing a PKI, cryptographic signatures, and a public blockchain

## Scope:



The goal of this project is to create a tool to upload documents and make them publicly verifiable, along with adding the option of selective disclosure for structured documents such as JSON files. Non-structured files would be salted and hashed, while structured files would be flattened and have each of their fields salted and hashed before constructing a merkle tree (or other selective disclosure cryptographic tool). Uploaded files would have their hashes or merkle trees stored on a publicly queryable database, allowing them to construct proofs for either their entire file, or individual fields. To ensure the uploaded documents cannot be tampered with, hashes or merkle roots will be committed to a public blockchain, likely a layer 2 chain based off of the etherium blockchain. When an uploader sends a file to be committed, they will receive a "receipt" file which contains an ID for the document, it's location on the public blockchain, and, if a selectively disclosable document, the flattened fields and their salts, necessary for actually proving your proofs. This information will also be stored on an off-chain database along with some data about the uploader and the merkle tree leaves if it is a structured document. The point of the system is not to store the files themselves, but to be able to prove that it either is the file you say it is, or that the fields of the document are what you claim them to be. Along with the system, a report on the technologies used, the reasons for their selection, what could've been done differently, and what could be added to the project, will be written.


## Deliverables: 
1. A cryptographic system which allows users to upload documents to a server which hashes them and/or builds a merkle tree from them, then anchors the documents to a public blockchain to ensure strong tamper resistance. The users are then given a file which can be used to prove a given document is the same as the one that was committed, and (if it is a structured document that is turned into a merkle tree) allows selective disclosure of individual document fields.
2. A report of the technology used for this project, along with why the technologies were chosen, what other options there were, and what would an expansion of the product look like.
3. Basic instructions on using/interacting with the project.
