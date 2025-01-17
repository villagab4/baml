import {
  TextDocuments,
  Diagnostic,
  InitializeParams,
  InitializeResult,
  CodeActionKind,
  CodeActionParams,
  HoverParams,
  CompletionItem,
  CompletionParams,
  DeclarationParams,
  RenameParams,
  DocumentFormattingParams,
  DidChangeConfigurationNotification,
  Connection,
  DocumentSymbolParams,
  TextDocumentSyncKind,
  CodeLensParams,
  Command,
  Position,
  Range,
  CodeLens,
  DidChangeWatchedFilesNotification,
} from 'vscode-languageserver'
import { URI } from 'vscode-uri'

import debounce from 'lodash/debounce'
import { createConnection, IPCMessageReader, IPCMessageWriter } from 'vscode-languageserver/node'
import { TextDocument } from 'vscode-languageserver-textdocument'

import * as MessageHandler from './lib/MessageHandler'
import type { LSOptions, LSSettings } from './lib/types'
import { getVersion, getEnginesVersion, getCliVersion } from './lib/wasm/internals'
import { BamlDirCache } from './file/fileCache'
import { LinterInput } from './lib/wasm/lint'
import { cliBuild, cliVersion } from './baml-cli'
import { ParserDatabase, TestRequest } from '@baml/common'
import generate_test_file from './lib/wasm/generate_test_file'
import { FileChangeType, workspace } from 'vscode'
import fs from 'fs'

const packageJson = require('../../package.json') // eslint-disable-line
function getConnection(options?: LSOptions): Connection {
  let connection = options?.connection
  if (!connection) {
    connection = process.argv.includes('--stdio')
      ? createConnection(process.stdin, process.stdout)
      : createConnection(new IPCMessageReader(process), new IPCMessageWriter(process))
  }
  return connection
}

let hasCodeActionLiteralsCapability = false
let hasConfigurationCapability = true

type BamlConfig = {
  path?: string
  trace: {
    server: string
  }
}
let config: BamlConfig | null = null

/**
 * Starts the language server.
 *
 * @param options Options to customize behavior
 */
export function startServer(options?: LSOptions): void {
  console.log('Server-side -- startServer()')
  // Source code: https://github.com/microsoft/vscode-languageserver-node/blob/main/server/src/common/server.ts#L1044
  const connection: Connection = getConnection(options)

  console.log = connection.console.log.bind(connection.console)
  console.error = connection.console.error.bind(connection.console)

  console.log('Starting Baml Language Server...')

  const documents: TextDocuments<TextDocument> = new TextDocuments(TextDocument)
  const bamlCache = new BamlDirCache()

  connection.onInitialize((params: InitializeParams) => {
    // Logging first...

    connection.console.info(
      // eslint-disable-next-line
      `Extension '${packageJson?.name}': ${packageJson?.version}`,
    )
    connection.console.info(`Using 'baml-wasm': ${getVersion()}`)
    const prismaEnginesVersion = getEnginesVersion()

    // ... and then capabilities of the language server
    const capabilities = params.capabilities

    hasCodeActionLiteralsCapability = Boolean(capabilities?.textDocument?.codeAction?.codeActionLiteralSupport)
    hasConfigurationCapability = Boolean(capabilities?.workspace?.configuration)

    const result: InitializeResult = {
      capabilities: {
        textDocumentSync: TextDocumentSyncKind.Full,
        definitionProvider: true,
        documentFormattingProvider: false,
        // completionProvider: {
        //   resolveProvider: false,
        //   triggerCharacters: ['@', '"', '.'],
        // },
        hoverProvider: true,
        renameProvider: false,
        documentSymbolProvider: true,
        codeLensProvider: {
          resolveProvider: true,
        },
        workspace: {
          fileOperations: {
            didCreate: {
              filters: [
                {
                  scheme: 'file',
                  pattern: {
                    glob: '**/*.{baml, json}',
                  },
                },
              ],
            },
            didDelete: {
              filters: [
                {
                  scheme: 'file',
                  pattern: {
                    glob: '**/*.{baml, json}',
                  },
                },
              ],
            },
            didRename: {
              filters: [
                {
                  scheme: 'file',
                  pattern: {
                    glob: '**/*.{baml, json}',
                  },
                },
              ],
            },
          }
        }
      },
    }

    const hasWorkspaceFolderCapability = !!(
      capabilities.workspace && !!capabilities.workspace.workspaceFolders
    );
    if (hasWorkspaceFolderCapability) {
      result.capabilities.workspace = {
        workspaceFolders: {
          supported: true
        }
      };
    }

    // if (hasCodeActionLiteralsCapability) {
    //   result.capabilities.codeActionProvider = {
    //     codeActionKinds: [CodeActionKind.QuickFix],
    //   }
    // }

    return result
  })

  connection.onInitialized(() => {
    console.log('initialized')

    if (hasConfigurationCapability) {
      // Register for all configuration changes.
      // eslint-disable-next-line @typescript-eslint/no-floating-promises
      connection.client.register(DidChangeConfigurationNotification.type)
      connection.client.register(DidChangeWatchedFilesNotification.type)
    }
  })

  // The global settings, used when the `workspace/configuration` request is not supported by the client or is not set by the user.
  // This does not apply to VS Code, as this client supports this setting.
  // const defaultSettings: LSSettings = {}
  // let globalSettings: LSSettings = defaultSettings // eslint-disable-line

  // Cache the settings of all open documents
  const documentSettings: Map<string, Thenable<LSSettings>> = new Map<string, Thenable<LSSettings>>()

  const getConfig = async () => {
    try {
      const configResponse = await connection.workspace.getConfiguration('baml')
      console.log('configResponse ' + JSON.stringify(configResponse, null, 2))
      config = configResponse as BamlConfig
    } catch (e: any) {
      if (e instanceof Error) {
        console.log('Error getting config' + e.message + ' ' + e.stack)
      } else {
        console.log('Error getting config' + e)
      }
    }
  }

  function getLanguageExtension(uri: string): string | undefined {
    const languageExtension = uri.split('.').pop()
    if (!languageExtension) {
      console.log('Could not find language extension for ' + uri)
      return
    }
    return languageExtension
  }

  connection.onDidChangeWatchedFiles(async (params) => {
    console.log('onDidChangeWatchedFiles ' + JSON.stringify(params, null, 2))
    params.changes.forEach((change) => {
      const uri = change.uri
      const languageExtension = getLanguageExtension(uri)
      if (!languageExtension) {
        return
      }
      const textDocument = TextDocument.create(uri, languageExtension, 1, '')
      bamlCache.refreshDirectory(textDocument)
    }
    )
    if (params.changes.length > 0) {
      const uri = params.changes[0].uri
      const languageExtension = getLanguageExtension(uri)
      if (!languageExtension) {
        return
      }
      const textDocument = TextDocument.create(uri, languageExtension, 1, '')
      validateTextDocument(textDocument)
    }
  });


  connection.onDidChangeConfiguration((_change) => {
    getConfig()
    if (hasConfigurationCapability) {
      // Reset all cached document settings
      documentSettings.clear()
    } else {
      // globalSettings = <LSSettings>(change.settings.prisma || defaultSettings) // eslint-disable-line @typescript-eslint/no-unsafe-member-access
    }

    // Revalidate all open prisma schemas
    documents.all().forEach(debouncedValidateTextDocument) // eslint-disable-line @typescript-eslint/no-misused-promises
  })

  documents.onDidOpen((e) => {
    try {
      // TODO: revalidate if something changed
      bamlCache.refreshDirectory(e.document)
      bamlCache.addDocument(e.document)
      debouncedValidateTextDocument(e.document)
    } catch (e: any) {
      if (e instanceof Error) {
        console.log('Error opening doc' + e.message + ' ' + e.stack)
      } else {
        console.log('Error opening doc' + e)
      }
    }
  })


  // Note: VS Code strips newline characters from the message
  function showErrorToast(errorMessage: string): void {
    connection.window
      .showErrorMessage(errorMessage, {
        title: 'Show Details',
      })
      .then((item) => {
        if (item?.title === 'Show Details') {
          connection.sendNotification('baml/showLanguageServerOutput')
        }
      })
  }

  function generateTestFile(test_request: TestRequest) {
    try {
      const { cache, root_path: rootPath } = bamlCache.lastBamlDir
      if (!rootPath || !cache) {
        console.error('Could not find root path')
        connection.sendNotification('baml/message', {
          type: 'error',
          message: 'Could not find a baml_src directory for root path',
        })
        return
      }
      const srcDocs = cache.getDocuments()
      const linterInput: LinterInput = {
        root_path: rootPath.fsPath,
        files: srcDocs.map(({ path, doc }) => {
          return {
            path,
            content: doc.getText(),
          }
        }),
      }

      if (srcDocs.length === 0) {
        console.log('No BAML files found in the workspace.')
        connection.sendNotification('baml/message', {
          type: 'warn',
          message: 'Unable to find BAML files. See Output panel -> BAML Language Server for more details.',
        })
      }
      const response = MessageHandler.handleGenerateTestFile(srcDocs, linterInput, test_request, showErrorToast)
      if (response.status === 'ok') {
        return response.content
      } else {
        showErrorToast(response.message)
      }
    } catch (e: any) {
      if (e instanceof Error) {
        console.log('Error generating test file' + e.message + ' ' + e.stack)
      } else {
        console.log('Error generating test file' + e)
      }
    }
  }

  // TODO: dont actually debounce for now or strange out of sync things happen..
  // so we currently set to 0
  const debouncedSetDb = debounce((rootPath: URI, db: ParserDatabase) => {
    void connection.sendRequest('set_database', { rootPath: rootPath.fsPath, db })
  }, 0, {
    maxWait: 4000,
    leading: true,
    trailing: true,
  });


  function validateTextDocument(textDocument: TextDocument) {
    try {
      const rootPath = bamlCache.getBamlDir(textDocument)
      if (!rootPath) {
        return
      }

      const srcDocs = bamlCache.getDocuments(textDocument)

      if (srcDocs.length === 0) {
        console.log(`No BAML files found in the workspace. ${rootPath}`)
        connection.sendNotification('baml/message', {
          type: 'warn',
          message: `Empty baml_src directory found: ${rootPath.fsPath}. See Output panel -> BAML Language Server for more details.`,
        })
        return
      }

      const response = MessageHandler.handleDiagnosticsRequest(rootPath, srcDocs, showErrorToast)
      for (const [uri, diagnosticList] of response.diagnostics) {
        void connection.sendDiagnostics({ uri, diagnostics: diagnosticList })
      }

      bamlCache.addDatabase(rootPath, response.state)
      if (response.state) {
        const filecache = bamlCache.getFileCache(textDocument)
        if (filecache) {
          filecache.setDB(response.state)
        } else {
          console.error('Could not find file cache for ' + textDocument.uri)
        }

        debouncedSetDb(rootPath, response.state)
      } else {
        void connection.sendRequest('rm_database', rootPath)
      }
    } catch (e: any) {
      if (e instanceof Error) {
        console.log('Error validating doc' + e.message + ' ' + e.stack)
      } else {
        console.log('Error validating doc' + e)
      }
    }
  }

  const debouncedValidateTextDocument = debounce(validateTextDocument, 400, {
    maxWait: 4000,
    leading: true,
    trailing: true,
  })
  const debouncedValidateCodelens = debounce(validateTextDocument, 1000, {
    maxWait: 4000,
    leading: true,
    trailing: true,
  });

  documents.onDidChangeContent((change: { document: TextDocument }) => {
    const textDocument = change.document;
    const rootPath = bamlCache.getBamlDir(textDocument)
    if (!rootPath) {
      console.error('Could not find root path for ' + textDocument.uri)
      connection.sendNotification('baml/message', {
        type: 'error',
        message: 'Could not find a baml_src directory for ' + textDocument.uri.toString(),
      })
      return
    }
    // add the document to the cache
    // we want to do this since the doc may not be in disk (it's in vscode memory).
    // If we try to just load docs from disk we will have outdated info.
    try {
      bamlCache.addDocument(textDocument)
    } catch (e) {
      console.log("Error adding document to cache " + e)
    }

    debouncedValidateTextDocument(textDocument)
  })

  const debouncedCLIBuild = debounce(cliBuild, 1000, {
    leading: true,
    trailing: true,
  })

  documents.onDidSave((change: { document: TextDocument }) => {
    try {
      const cliPath = config?.path || 'baml'
      let bamlDir = bamlCache.getBamlDir(change.document)
      if (!bamlDir) {
        console.error(
          'Could not find baml_src dir for ' + change.document.uri + '. Make sure your baml files are in baml_src dir',
        )
        return
      }


      debouncedCLIBuild(cliPath, bamlDir, showErrorToast, () => {
        connection.sendNotification('baml/message', {
          type: 'info',
          message: 'Generated BAML client successfully!',
        })
      })
    } catch (e: any) {
      if (e instanceof Error) {
        console.log('Error saving doc' + e.message + ' ' + e.stack)
      } else {
        console.log('Error saving doc' + e)
      }
    }
  })

  function getDocument(uri: string): TextDocument | undefined {
    return documents.get(uri)
  }

  connection.onDefinition((params: DeclarationParams) => {
    const doc = getDocument(params.textDocument.uri)
    if (doc) {
      const db = bamlCache.getFileCache(doc)
      if (db) {
        return MessageHandler.handleDefinitionRequest(db, doc, params)
      } else if (doc.languageId === 'python') {
        const db = bamlCache.lastBamlDir?.cache
        console.log(` python: ${doc.uri} files: ${db?.getDocuments().length}`)
        if (db) {
          return MessageHandler.handleDefinitionRequest(db, doc, params)
        }
      }
    }
  })

  // connection.onCompletion((params: CompletionParams) => {
  //   const doc = getDocument(params.textDocument.uri)
  //   if (doc) {
  //     return MessageHandler.handleCompletionRequest(params, doc, showErrorToast)
  //   }
  // })

  // This handler resolves additional information for the item selected in the completion list.
  // connection.onCompletionResolve((completionItem: CompletionItem) => {
  //   return MessageHandler.handleCompletionResolveRequest(completionItem)
  // })

  connection.onHover((params: HoverParams) => {
    const doc = getDocument(params.textDocument.uri)
    if (doc) {
      const db = bamlCache.getFileCache(doc)
      if (db) {
        return MessageHandler.handleHoverRequest(db, doc, params)
      }
    }
  })

  connection.onCodeLens((params: CodeLensParams) => {
    const document = getDocument(params.textDocument.uri)
    const codeLenses: CodeLens[] = []
    if (!document) {
      return codeLenses
    }
    bamlCache.addDocument(document)
    // Must be separate from the other validateText since we don't want to get stale in our code lenses.
    debouncedValidateCodelens(document)

    const db = bamlCache.getParserDatabase(document);
    const docFsPath = URI.parse(document.uri).fsPath;
    const baml_dir = bamlCache.getBamlDir(document);
    if (!db) {
      console.log('No db for ' + document.uri + ". There may be a linter error or out of sync file");
      return codeLenses
    }

    const functionNames = db.functions.filter((x) => x.name.source_file === docFsPath).map((f) => f.name)
    const position: Position = document.positionAt(0);
    functionNames.forEach((name) => {
      const range = Range.create(document.positionAt(name.start), document.positionAt(name.end))
      const command: Command = {
        title: '▶️ Open Playground',
        command: 'baml.openBamlPanel',
        arguments: [
          {
            projectId: baml_dir?.fsPath || '',
            functionName: name.value,
            showTests: true,
          },
        ],
      }
      codeLenses.push({
        range,
        command
      });
    })

    const implNames = db.functions
      .flatMap((f) =>
        f.impls.map((i) => {
          return {
            value: i.name.value,
            start: i.name.start,
            end: i.name.end,
            source_file: i.name.source_file,
            prompt_key: i.prompt_key,
            function: f.name.value,
          }
        }),
      )
      .filter((x) => x.source_file === docFsPath)

    implNames.forEach((name) => {
      codeLenses.push(
        {
          range: (Range.create(document.positionAt(name.start), document.positionAt(name.end))),
          command: {
            title: '▶️ Open Playground',
            command: 'baml.openBamlPanel',
            arguments: [
              {
                projectId: baml_dir?.fsPath || '',
                functionName: name.function,
                implName: name.value,
                showTests: true,
              },
            ],
          }
        }
      )
      codeLenses.push(
        {
          range: Range.create(document.positionAt(name.prompt_key.start), document.positionAt(name.prompt_key.end)),
          command: {
            title: '▶️ Open Live Preview',
            command: 'baml.openBamlPanel',
            arguments: [
              {
                projectId: baml_dir?.fsPath || '',
                functionName: name.function,
                implName: name.value,
                showTests: false,
              },
            ],
          },
        },
      )

    })

    const testCases = db.functions
      .flatMap((f) =>
        f.test_cases.map((t) => {
          return {
            value: t.name.value,
            start: t.name.start,
            end: t.name.end,
            source_file: t.name.source_file,
            function: f.name.value,
          }
        }),
      )
      .filter((x) => x.source_file === docFsPath)
    testCases.forEach((name) => {
      const range = Range.create(document.positionAt(name.start), document.positionAt(name.end))
      const command: Command = {
        title: '▶️ Open Playground',
        command: 'baml.openBamlPanel',
        arguments: [
          {
            projectId: baml_dir?.fsPath || '',
            functionName: name.function,
            testCaseName: name.value,
            showTests: true,
          },
        ],
      }
      codeLenses.push({
        range,
        command
      });

    })

    return codeLenses

    // return [];
  })


  // connection.onDocumentFormatting((params: DocumentFormattingParams) => {
  //   const doc = getDocument(params.textDocument.uri)
  //   if (doc) {
  //     return MessageHandler.handleDocumentFormatting(params, doc, showErrorToast)
  //   }
  // })

  // connection.onCodeAction((params: CodeActionParams) => {
  //   const doc = getDocument(params.textDocument.uri)
  //   if (doc) {
  //     return MessageHandler.handleCodeActions(params, doc, showErrorToast)
  //   }
  // })

  // connection.onRenameRequest((params: RenameParams) => {
  //   const doc = getDocument(params.textDocument.uri)
  //   if (doc) {
  //     return MessageHandler.handleRenameRequest(params, doc)
  //   }
  // })

  connection.onDocumentSymbol((params: DocumentSymbolParams) => {
    const doc = getDocument(params.textDocument.uri)
    if (doc) {
      const db = bamlCache.getFileCache(doc)
      if (db) {
        let symbols = MessageHandler.handleDocumentSymbol(db, params, doc)
        return symbols
      }
    }
  })

  connection.onRequest('getDefinition', ({ sourceFile, name }: { sourceFile: string; name: string }) => {
    const fileCache = bamlCache.getCacheForUri(sourceFile)
    if (fileCache) {
      let match = fileCache.define(name)
      if (match) {
        return {
          targetUri: match.uri.toString(),
          targetRange: match.range,
          targetSelectionRange: match.range,
        }
      }
    }
  })

  connection.onRequest('cliVersion', async () => {
    console.log('Checking baml version at ' + config?.path)
    try {
      const res = await new Promise<string>((resolve, reject) => {
        cliVersion(config?.path || 'baml', reject, (ver) => {
          resolve(ver)
        })
      })

      return res
    } catch (e: any) {
      if (e instanceof Error) {
        console.log('Error getting cli version' + e.message + ' ' + e.stack)
      } else {
        console.log('Error getting cli version' + e)
      }
      return undefined
    }
  })

  connection.onRequest('generatePythonTests', (params: TestRequest) => {
    return generateTestFile(params)
  })

  connection.onRequest("saveFile", async (params: {
    filepath: string;
  }) => {
    console.log("saveFile" + JSON.stringify(params, null, 2));
    const uri = URI.parse(params.filepath);
    const document = getDocument(uri.toString());

    if (!document) {
      console.log("Could not find document for " + uri.toString());
      return;
    }
    try {
      fs.writeFileSync(uri.fsPath, document.getText());
    } catch (e) {
      console.error("Error writing file " + e);
    }
  })

  console.log('Server-side -- listening to connection')
  // Make the text document manager listen on the connection
  // for open, change and close text document events
  documents.listen(connection)

  connection.listen()
}
