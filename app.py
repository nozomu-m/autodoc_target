from flask import Flask, request, jsonify
from flask_jwt_extended import JWTManager, create_access_token, jwt_required, get_jwt_identity
import json
import os

app = Flask(__name__)
app.config['JWT_SECRET_KEY'] = 'your_jwt_secret_key'
jwt = JWTManager(app)

USER_FILE = 'users.json'
SCHEDULE_FILE = 'schedules.json'

# ユーザーデータをロード
def load_users():
    if not os.path.exists(USER_FILE):
        return []
    with open(USER_FILE, 'r') as file:
        return json.load(file)

# ユーザーデータを保存
def save_users(users):
    with open(USER_FILE, 'w') as file:
        json.dump(users, file)

# スケジュールデータをロード
def load_schedules():
    if not os.path.exists(SCHEDULE_FILE):
        return []
    with open(SCHEDULE_FILE, 'r') as file:
        return json.load(file)

# スケジュールデータを保存
def save_schedules(schedules):
    with open(SCHEDULE_FILE, 'w') as file:
        json.dump(schedules, file)

# ユーザー登録
@app.route('/register', methods=['POST'])
def register():
    data = request.get_json()
    username = data.get('username')
    password = data.get('password')
    users = load_users()
    if any(u['username'] == username for u in users):
        return jsonify({"msg": "Username already exists"}), 400
    new_user = {
        "id": len(users) + 1,
        "username": username,
        "password": password
    }
    users.append(new_user)
    save_users(users)
    return jsonify({"msg": "User registered successfully"}), 201

# ユーザーログイン
@app.route('/login', methods=['POST'])
def login():
    data = request.get_json()
    username = data.get('username')
    password = data.get('password')
    users = load_users()
    user = next((u for u in users if u['username'] == username and u['password'] == password), None)
    if user:
        access_token = create_access_token(identity=user['id'])
        return jsonify(access_token=access_token)
    return jsonify({"msg": "Invalid credentials"}), 401

# スケジュール追加
@app.route('/schedules', methods=['POST'])
@jwt_required()
def add_schedule():
    data = request.get_json()
    current_user_id = get_jwt_identity()
    schedules = load_schedules()
    new_schedule = {
        "id": len(schedules) + 1,
        "user_id": current_user_id,
        "title": data['title'],
        "date": data['date']
    }
    schedules.append(new_schedule)
    save_schedules(schedules)
    return jsonify({"msg": "Schedule added"}), 201

# スケジュール閲覧
@app.route('/schedules', methods=['GET'])
@jwt_required()
def get_schedules():
    current_user_id = get_jwt_identity()
    schedules = load_schedules()
    user_schedules = [s for s in schedules if s['user_id'] == current_user_id]
    return jsonify(user_schedules)

# スケジュール削除
@app.route('/schedules/<int:schedule_id>', methods=['DELETE'])
@jwt_required()
def delete_schedule(schedule_id):
    current_user_id = get_jwt_identity()
    schedules = load_schedules()
    schedule = next((s for s in schedules if s['id'] == schedule_id and s['user_id'] == current_user_id), None)
    if schedule:
        schedules.remove(schedule)
        save_schedules(schedules)
        return jsonify({"msg": "Schedule deleted"})
    return jsonify({"msg": "Schedule not found"}), 404

# 友人のスケジュールを見る
@app.route('/friends_schedules/<int:friend_id>', methods=['GET'])
@jwt_required()
def get_friend_schedules(friend_id):
    schedules = load_schedules()
    friend_schedules = [s for s in schedules if s['user_id'] == friend_id]
    return jsonify(friend_schedules)

if __name__ == '__main__':
    app.run(debug=True)

