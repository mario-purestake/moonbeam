import { Client, MessageEmbed, Message } from "discord.js";
import Web3 from "web3";


const TOKEN_DECIMAL = 18n;
const FAUCET_SEND_INTERVAL = 1; // hours
const EMBED_COLOR_CORRECT = 0x642f95;
const EMBED_COLOR_ERROR = 0xc0392b;

const params = {
	// Discord app information
	DISCORD_TOKEN: process.env.DISCORD_TOKEN,
	DISCORD_CHANNEL: process.env.DISCORD_CHANNEL,

	// Web3 RPC access
	RPC_URL: process.env.RPC_URL,
	ACCOUNT_KEY: process.env.ACCOUNT_KEY,

	// Token distribution
	TOKEN_COUNT: BigInt(process.env.TOKEN_COUNT || 10),
}

Object.keys(params).forEach(param => {
	if (!params[param]) {
		console.log(`Missing ${param} env variables`);
		process.exit(1);
	}
})

const web3Api = new Web3(params.RPC_URL);

console.log(`Starting bot...`);
console.log(`Connecting web3 to ${params.RPC_URL}...`);

const client: Client = new Client();
const receivers: { [author: string]: number } = {};

client.on("ready", () => {
	console.log(`Logged in as ${client.user.tag}!`);
});

/**
 * Returns the approximated remaining time until being able to request tokens again.
 * @param {number} lastTokenRequestMoment Last moment in which the user requested funds
 */
const nextAvailableToken = (lastTokenRequestMoment: number) => {
	// how many ms there are in minutes/hours
	const msPerMinute = 60 * 1000;
	const msPerHour = msPerMinute * 60;

	// when the author of the message will be able to request more tokens
	const availableAt = lastTokenRequestMoment + (FAUCET_SEND_INTERVAL * msPerHour);
	// remaining time until able to request more tokens
	let remain = availableAt - Date.now();

	if (remain < msPerMinute) {
		return `${Math.round(remain / 1000)} second(s)`;
	}
	else if (remain < msPerHour) {
		return `${Math.round(remain / msPerMinute)} minute(s)`;
	}
	else {
		return `${Math.round(remain / msPerHour)} hour(s)`;
	}
}

/**
 * Checks that the address follows the H160 adress format
 * @param {string} address Address to check
 * @param {Message} msg Received discord message object
 */
const checkH160AddressIsCorrect = (address: string, msg: Message) => {
	let addressIsCorrect = true;

	// slice address if defined in hexadecimal
	if (address.startsWith("0x")) {
		address = address.slice(2);
	}

	// check that address is 40 characters long
	if (address.length != 40) {
		addressIsCorrect = false
	}

	// check that address only contains alphanumerical characters
	if (!address.match(/^[a-z0-9]+$/i)) {
		addressIsCorrect = false;
	}


	// resolve if address was not correct
	if (addressIsCorrect === false) {
		const errorEmbed = new MessageEmbed()
			.setColor(EMBED_COLOR_ERROR)
			.setTitle("Invalid address")
			.setFooter("Addresses must follow the H160 address format");
	
		// send message to channel
		msg.channel.send(errorEmbed);
	}

	return addressIsCorrect;
}

/**
 * Action for the bot for the pattern "!faucet send <h160_addr>", that 
 * sends funds to the indicated account.
 * @param {Message} msg Received discord message object
 * @param {string} authorId Author ID of the message
 * @param {string} messageContent Content of the message
 */
const botActionFaucetSend = async (msg: Message, authorId: string, messageContent: string) => {
	if (receivers[authorId] > Date.now() - FAUCET_SEND_INTERVAL * 3600 * 1000) {
		const errorEmbed = new MessageEmbed()
			.setColor(EMBED_COLOR_ERROR)
			.setTitle(`You already received tokens!`)
			.addField("Remaining time", `You still need to wait ${nextAvailableToken(receivers[authorId])} to receive more tokens`)
			.setFooter("Funds transactions are limited to once per hour");

		msg.channel.send(errorEmbed);
		return;
	}

	let address = messageContent.slice("!faucet send".length).trim();
	if (address.startsWith("0x")) { address = address.slice(2); }

	// check address and send alert msg and return if bad formatted
	if (!checkH160AddressIsCorrect(address, msg)) return;

	// update user last fund retrieval
	receivers[authorId] = Date.now();

	await web3Api.eth.sendSignedTransaction(
		(
			await web3Api.eth.accounts.signTransaction(
				{
					value: `${params.TOKEN_COUNT * (10n ** TOKEN_DECIMAL)}`,
					gasPrice: "0x01",
					gas: "0x21000",
					to: `0x${address}`,
				},
				params.ACCOUNT_KEY
			)
		).rawTransaction
	);
	const accountBalance = BigInt(await web3Api.eth.getBalance(`0x${address}`));

	const fundsTransactionEmbed = new MessageEmbed()
		.setColor(EMBED_COLOR_CORRECT)
		.setTitle("Transaction of funds")
		.addField("To account", `0x${address}`, true)
		.addField("Amount sent", `${params.TOKEN_COUNT} DEV`, true)
		.addField("Current account balance", `${accountBalance / (10n ** TOKEN_DECIMAL)} DEV`)
		.setFooter("Funds transactions are limited to once per hour");

	msg.channel.send(fundsTransactionEmbed);
}

/**
 * Action for the bot for the pattern "!balance <h160_addr>", that 
 * checks the balance of the indicated account.
 * @param {Message} msg Received discord message object
 * @param {string} messageContent Content of the message
 */
const botActionBalance = async (msg: Message, messageContent: string) => {
	let address = messageContent.slice("!balance".length).trim();
	if (address.startsWith("0x")) { address = address.slice(2); }

	// check address and send alert msg and return if bad formatted
	if (!checkH160AddressIsCorrect(address, msg)) return;

	const accountBalance = BigInt(await web3Api.eth.getBalance(`0x${address}`));

	const balanceEmbed = new MessageEmbed()
		.setColor(EMBED_COLOR_CORRECT)
		.setTitle("Account Balance")
		.addField("Account", `0x${address}`, true)
		.addField("Balance", `${accountBalance / (10n ** TOKEN_DECIMAL)} DEV`, true);

	msg.channel.send(balanceEmbed);
}

const onReceiveMessage = async (msg: Message) => {
	const authorId = msg && msg.author && msg.author.id;
	const messageContent = msg && msg.content;
	const channelId = msg && msg.channel && msg.channel.id;

	if (!messageContent || !authorId || channelId != params.DISCORD_CHANNEL) {
		return;
	}

	if (messageContent.startsWith("!faucet send")) {
		await botActionFaucetSend(msg, authorId, messageContent);
	}
	else if (messageContent.startsWith("!balance")) {
		await botActionBalance(msg, messageContent);
	}
};

client.on("message", onReceiveMessage);

client.login(params.DISCORD_TOKEN);
