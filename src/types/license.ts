export interface ActivationRequest {
	appId: string;
	appVersion: string;
	userId: string;
	deviceFingerprint: string;
	deviceHint: string;
	requestTime: string;
}

export interface LicenseStatus {
	isActivated: boolean;
	isValid: boolean;
	reason: string;
	checkedAt: string;
	userId: string | null;
	licenseId: string | null;
	issuedAt: string | null;
	expiresAt: string | null;
	maxVersion: string | null;
	features: string[];
	currentVersion: string;
	deviceHint: string;
	deviceFingerprint: string;
}

export interface SignedLicense {
	licenseId: string;
	userId: string;
	deviceFingerprint: string;
	issuedAt: string;
	expiresAt: string;
	maxVersion: string;
	features: string[];
	signature: string;
}

export interface SignerStatus {
	isConfigured: boolean;
	isAllowed: boolean;
	reason: string;
	publicKey: string | null;
	currentDeviceFingerprint: string | null;
	currentDeviceHint: string;
}
