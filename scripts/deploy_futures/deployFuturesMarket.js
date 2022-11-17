"use strict";
// Execute this script with the command: yarn deploy-futures
// This script will deploy the futures market and launch a single test trade
// Deployment is made from the perspective of the deployment authority specified in localConfig.json
// This script assumes that a valid network has been deployed with associated data saved to protonet.json
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (_) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
exports.__esModule = true;
// IMPORTS
// LOCAL
var protonet_json_1 = __importDefault(require("../../configs/protonet.json"));
var localConfig_json_1 = __importDefault(require("./localConfig.json"));
var utils_1 = require("./utils");
// EXTERNAL
var fermi_js_sdk_1 = require("fermi-js-sdk");
var tenex_js_sdk_1 = require("tenex-js-sdk");
var DeploymentBuilder = /** @class */ (function () {
    function DeploymentBuilder(privateKey, client) {
        this.privateKey = privateKey;
        this.client = client;
    }
    DeploymentBuilder.prototype.sendCreateAsset = function (dummy) {
        return __awaiter(this, void 0, void 0, function () {
            var signedTransaction, result;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, tenex_js_sdk_1.TenexUtils.buildSignedTransaction(
                        /* request */ tenex_js_sdk_1.TenexTransaction.buildCreateAssetRequest(dummy), 
                        /* senderPrivKey */ this.privateKey, 
                        /* recentBlockDigest */ undefined, 
                        /* fee */ undefined, 
                        /* client */ this.client)];
                    case 1:
                        signedTransaction = _a.sent();
                        console.log('Sending asset creation tranasction');
                        return [4 /*yield*/, this.client.sendAndConfirmTransaction(signedTransaction)];
                    case 2:
                        result = _a.sent();
                        console.log('result=', result);
                        fermi_js_sdk_1.FermiUtils.checkSubmissionResult(result);
                        return [2 /*return*/];
                }
            });
        });
    };
    DeploymentBuilder.prototype.sendCreateMarketplaceRequest = function (quoteAssetId) {
        return __awaiter(this, void 0, void 0, function () {
            var signedTransaction, result;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, tenex_js_sdk_1.TenexUtils.buildSignedTransaction(
                        /* request */ tenex_js_sdk_1.TenexTransaction.buildCreateMarketplaceRequest(quoteAssetId), 
                        /* senderPrivKey */ this.privateKey, 
                        /* recentBlockDigest */ undefined, 
                        /* fee */ undefined, 
                        /* client */ this.client)];
                    case 1:
                        signedTransaction = _a.sent();
                        console.log('Sending create marketplace tranasction');
                        return [4 /*yield*/, this.client.sendAndConfirmTransaction(signedTransaction)];
                    case 2:
                        result = _a.sent();
                        console.log('result=', result);
                        fermi_js_sdk_1.FermiUtils.checkSubmissionResult(result);
                        return [2 /*return*/];
                }
            });
        });
    };
    DeploymentBuilder.prototype.sendCreateMarketRequest = function (baseAssetId) {
        return __awaiter(this, void 0, void 0, function () {
            var signedTransaction, result;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, tenex_js_sdk_1.TenexUtils.buildSignedTransaction(
                        /* request */ tenex_js_sdk_1.TenexTransaction.buildCreateMarketRequest(baseAssetId), 
                        /* senderPrivKey */ this.privateKey, 
                        /* recentBlockDigest */ undefined, 
                        /* fee */ undefined, 
                        /* client */ this.client)];
                    case 1:
                        signedTransaction = _a.sent();
                        console.log('Sending create market tranasction');
                        return [4 /*yield*/, this.client.sendAndConfirmTransaction(signedTransaction)];
                    case 2:
                        result = _a.sent();
                        console.log('result=', result);
                        fermi_js_sdk_1.FermiUtils.checkSubmissionResult(result);
                        return [2 /*return*/];
                }
            });
        });
    };
    DeploymentBuilder.prototype.sendUpdatePricesRequest = function (latestPrices) {
        return __awaiter(this, void 0, void 0, function () {
            var signedTransaction, result;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, tenex_js_sdk_1.TenexUtils.buildSignedTransaction(
                        /* request */ tenex_js_sdk_1.TenexTransaction.buildUpdatePricesRequest(latestPrices), 
                        /* senderPrivKey */ this.privateKey, 
                        /* recentBlockDigest */ undefined, 
                        /* fee */ undefined, 
                        /* client */ this.client)];
                    case 1:
                        signedTransaction = _a.sent();
                        console.log('Sending update prices tranasction');
                        return [4 /*yield*/, this.client.sendAndConfirmTransaction(signedTransaction)];
                    case 2:
                        result = _a.sent();
                        console.log('result=', result);
                        fermi_js_sdk_1.FermiUtils.checkSubmissionResult(result);
                        return [2 /*return*/];
                }
            });
        });
    };
    DeploymentBuilder.prototype.sendAccountDepositRequest = function (quantity, marketAdmin) {
        return __awaiter(this, void 0, void 0, function () {
            var signedTransaction, result;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, tenex_js_sdk_1.TenexUtils.buildSignedTransaction(
                        /* request */ tenex_js_sdk_1.TenexTransaction.buildAccountDepositRequest(quantity, marketAdmin), 
                        /* senderPrivKey */ this.privateKey, 
                        /* recentBlockDigest */ undefined, 
                        /* fee */ undefined, 
                        /* client */ this.client)];
                    case 1:
                        signedTransaction = _a.sent();
                        console.log('Sending create market tranasction');
                        return [4 /*yield*/, this.client.sendAndConfirmTransaction(signedTransaction)];
                    case 2:
                        result = _a.sent();
                        console.log('result=', result);
                        fermi_js_sdk_1.FermiUtils.checkSubmissionResult(result);
                        return [2 /*return*/];
                }
            });
        });
    };
    DeploymentBuilder.prototype.sendFuturesLimitOrderRequest = function (baseAssetId, quoteAssetId, side, price, quantity, marketAdmin) {
        return __awaiter(this, void 0, void 0, function () {
            var signedTransaction, result;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, tenex_js_sdk_1.TenexUtils.buildSignedTransaction(
                        /* request */ tenex_js_sdk_1.TenexTransaction.buildFuturesLimitOrderRequest(baseAssetId, quoteAssetId, side, price, quantity, marketAdmin), 
                        /* senderPrivKey */ this.privateKey, 
                        /* recentBlockDigest */ undefined, 
                        /* fee */ undefined, 
                        /* client */ this.client)];
                    case 1:
                        signedTransaction = _a.sent();
                        console.log('Sending create market tranasction');
                        return [4 /*yield*/, this.client.sendAndConfirmTransaction(signedTransaction)];
                    case 2:
                        result = _a.sent();
                        console.log('result=', result);
                        fermi_js_sdk_1.FermiUtils.checkSubmissionResult(result);
                        return [2 /*return*/];
                }
            });
        });
    };
    return DeploymentBuilder;
}());
function main() {
    return __awaiter(this, void 0, void 0, function () {
        var authorities, deploymentAuthority, client, deployerPrivateKey, deployerPublicKey, deployer;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    console.log('config=', protonet_json_1["default"]);
                    authorities = Object.keys(protonet_json_1["default"]['authorities']);
                    deploymentAuthority = protonet_json_1["default"]['authorities'][authorities[localConfig_json_1["default"].deploymentAuthority]];
                    console.log('deploymentAuthority=', deploymentAuthority);
                    client = new fermi_js_sdk_1.FermiClient((0, utils_1.getJsonRpcUrl)(deploymentAuthority));
                    deployerPrivateKey = fermi_js_sdk_1.FermiUtils.hexToBytes(deploymentAuthority.private_key);
                    return [4 /*yield*/, fermi_js_sdk_1.FermiAccount.getPublicKey(deployerPrivateKey)];
                case 1:
                    deployerPublicKey = _a.sent();
                    deployer = new DeploymentBuilder(deployerPrivateKey, client);
                    console.log('Starting Deployment Now...');
                    return [4 /*yield*/, deployer.sendCreateAsset(/* dummy */ 0)];
                case 2:
                    _a.sent();
                    return [4 /*yield*/, deployer.sendCreateAsset(/* dummy */ 1)];
                case 3:
                    _a.sent();
                    return [4 /*yield*/, deployer.sendCreateMarketplaceRequest(/* quoteAssetId */ 1)];
                case 4:
                    _a.sent();
                    return [4 /*yield*/, deployer.sendCreateMarketRequest(/* baseAssetId */ 0)];
                case 5:
                    _a.sent();
                    return [4 /*yield*/, deployer.sendUpdatePricesRequest(/* latestPrices */ [1000000])];
                case 6:
                    _a.sent();
                    return [4 /*yield*/, deployer.sendAccountDepositRequest(
                        /* quantity */ 1000000, 
                        /* marketAdmin */ deployerPublicKey)];
                case 7:
                    _a.sent();
                    return [4 /*yield*/, deployer.sendFuturesLimitOrderRequest(
                        /* baseAssetId */ 0, 
                        /* quoteAssetId */ 1, 
                        /* side */ 1, 
                        /* price */ 1000, 
                        /* quantity */ 1000, 
                        /* admin */ deployerPublicKey)];
                case 8:
                    _a.sent();
                    console.log('Successfully Deployed And Tested!');
                    return [2 /*return*/];
            }
        });
    });
}
main();
