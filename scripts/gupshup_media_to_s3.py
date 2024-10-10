import requests
import boto3
import os

# Define your constants
SECRET_KEY = os.getenv("SECRET_KEY")
ALL = os.getenv("ALL") or "false"
API_GET_MEDIA_URL = f"https://todayhasbeen.com/api/v0.1/get_gupshup_medias/?secret_key={SECRET_KEY}&all={ALL}"
API_CHANGE_MEDIA_URL = f"https://todayhasbeen.com/api/v0.1/change_media_urls/?secret_key={SECRET_KEY}"
# API_GET_MEDIA_URL = f"http://127.0.0.1:8001/api/v0.1/get_gupshup_medias/?secret_key={SECRET_KEY}&all={ALL}"
# API_CHANGE_MEDIA_URL = f"http://127.0.0.1:8001/api/v0.1/change_media_urls/?secret_key={SECRET_KEY}"
S3_BUCKET_NAME = "thb-test"

# Initialize S3 client
s3 = boto3.client('s3', region_name='ap-south-1')

def fetch_media_data():
    """Fetch the media data from the API"""
    response = requests.get(API_GET_MEDIA_URL)
    if response.status_code == 200:
        return response.json()
    else:
        print(f"Failed to fetch media data. Status Code: {response.status_code}")
        return None

def download_media(media_url, post_id):
    """Download the media file from the given URL"""
    local_filename = f"media_{post_id}.jpg"  # Customize file name if needed
    response = requests.get(media_url, stream=True)
    if response.status_code == 200:
        with open(local_filename, 'wb') as file:
            for chunk in response.iter_content(chunk_size=8192):
                file.write(chunk)
        print(f"Downloaded media for post_id: {post_id}")
        return local_filename
    else:
        print(f"Failed to download media for post_id: {post_id}")
        return None

def upload_to_s3(file_path, post_id):
    """Upload the downloaded media to S3"""
    s3_key = f"uploads/{os.path.basename(file_path)}"
    try:
        s3.upload_file(file_path, S3_BUCKET_NAME, s3_key)
        s3_url = f"https://{S3_BUCKET_NAME}.s3.amazonaws.com/{s3_key}"
        print(f"Uploaded file to S3 for post_id: {post_id}")
        return s3_url
    except Exception as e:
        print(f"Failed to upload to S3 for post_id: {post_id}. Error: {e}")
        return None

def update_media_url(post_id, s3_url):
    """Update the media URL by calling the change media URL API"""
    payload = [{"post_id": post_id, "media_url": s3_url}]
    response = requests.post(API_CHANGE_MEDIA_URL, json=payload)
    if response.status_code == 200:
        print(f"Successfully updated media URL for post_id: {post_id}")
    else:
        print(f"Failed to update media URL for post_id: {post_id}. Status Code: {response.status_code}")

def process_media():
    """Main function to process all the media"""
    media_data = fetch_media_data()

    if media_data and media_data.get("success"):
        for item in media_data.get("data", []):
            post_id = item.get("post_id")
            media_url = item.get("media_url")

            # Step 1: Download media
            downloaded_file = download_media(media_url, post_id)
            if downloaded_file:
                # Step 2: Upload to S3
                s3_url = upload_to_s3(downloaded_file, post_id)

                if s3_url:
                    # Step 3: Update media URL via API
                    update_media_url(post_id, s3_url)

                # Cleanup downloaded file
                os.remove(downloaded_file)
    else:
        print("No media to process or failed to fetch media data.")

if __name__ == "__main__":
    process_media()
