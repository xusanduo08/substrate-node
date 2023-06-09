# Kitties

## Test

![image](https://github.com/xusanduo08/substrate-node/assets/17930163/c2fb1de2-c20c-4cb5-b1cb-ee87859b336f)


## Build

![image](https://github.com/xusanduo08/substrate-node/assets/17930163/b7ebe938-04e4-4597-a691-27790e39040f)

## Run

![image](https://github.com/xusanduo08/substrate-node/assets/17930163/621c9045-9b64-4bb8-84fa-d6e9ca87354e)

![image](https://github.com/xusanduo08/substrate-node/assets/17930163/93e58daa-defd-4917-86e0-1923246355ba)


## Upgrade

### V0->V1

升级前：版本为100，`create`方法不带有`name`参数
![image](https://github.com/xusanduo08/substrate-node/assets/17930163/b7326ae9-cf28-4cce-96bd-a2a0fdaaf9d4)


查询kitty_id = 0的kitty，如下：
![image](https://github.com/xusanduo08/substrate-node/assets/17930163/debedc25-709e-4944-84de-fc07d09bc8f8)


开始升级

![企业微信截图_16862803535154](https://github.com/xusanduo08/substrate-node/assets/17930163/10c48a71-2da9-496b-b1c4-ea08c0000db3)

升级完成
![企业微信截图_16862804756402](https://github.com/xusanduo08/substrate-node/assets/17930163/cd800e1d-4b59-4c61-9999-05485c53dcae)


查看刚刚创建的kitty，如下
![企业微信截图_16862805004720](https://github.com/xusanduo08/substrate-node/assets/17930163/ffe472bd-61ba-4021-af83-e39c1b0d6d68)

数据正确

### V0 -> V2

升级前：版本为100， `create`方法不带参数
![image](https://github.com/xusanduo08/substrate-node/assets/17930163/f2ce1b75-2545-4932-96be-79ef40c115cc)


创建一个kitty，然后查询kitty_id=0的数据，如下：
![image](https://github.com/xusanduo08/substrate-node/assets/17930163/760fb52c-5f2a-4c1c-8390-66f635efca18)


开始升级
![企业微信截图_16863045659408](https://github.com/xusanduo08/substrate-node/assets/17930163/d712be4c-bd3a-425a-98b5-c978721b2e63)

升级成功，左上角版本号已变为300
![企业微信截图_16863045892198](https://github.com/xusanduo08/substrate-node/assets/17930163/aa231f72-b4a6-4655-8612-474c9a4fcbf7)

再次查询刚刚创建的kitty，name字段已变为8字节，数据正确
![企业微信截图_16863046249602](https://github.com/xusanduo08/substrate-node/assets/17930163/3c20bfb2-2ff9-40e6-8e79-0e95e7518d65)



### V1 -> V2
升级前：版本为200，kitty带有name:[u8;4]属性

创建
![image](https://github.com/xusanduo08/substrate-node/assets/17930163/007ad78f-637c-4170-b122-0606d5922366)

查询
![image](https://github.com/xusanduo08/substrate-node/assets/17930163/75d67920-4a34-4b1f-9421-48479fd29b6f)

开始升级
![企业微信截图_16863012377126](https://github.com/xusanduo08/substrate-node/assets/17930163/968eb7e0-fa39-4d68-a166-ba3f5ce4ff3b)

升级成功，左上角版本号已经变为300
![企业微信截图_1686301277434](https://github.com/xusanduo08/substrate-node/assets/17930163/41de9ee9-5ff2-467a-91cf-023147559101)

再次查询刚刚创建的kitty，name字段已经变成8字节，数据正确
![image](https://github.com/xusanduo08/substrate-node/assets/17930163/7d73781a-21ca-40e8-b1db-cd1b7c2afc92)



