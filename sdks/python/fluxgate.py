import requests

class FluxGateClient:
    def __init__(self, api_key: str, base_url: str = "http://localhost:8080/v1"):
        self.api_key = api_key
        self.base_url = base_url
        self.headers = {"Authorization": f"Bearer {api_key}", "Content-Type": "application/json"}

    def process(self, prompt: str, session_id: str = None) -> dict:
        """Send prompt securely through FluxGate"""
        payload = {"prompt": prompt}
        if session_id:
            payload["session_id"] = session_id
        res = requests.post(f"{self.base_url}/process", json=payload, headers=self.headers)
        res.raise_for_status()
        return res.json()
